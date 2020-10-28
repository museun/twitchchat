use futures::{Stream, StreamExt};
use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};

use super::DecodeError;
use crate::{irc::IrcMessage, wait_for::Event, IntoOwned as _};

/// This trait is used to read a `String` from a `Stream` item
pub trait ReadMessage: Sized {
    /// Read a string from this item
    fn read_string(self) -> Result<String, DecodeError>;
}

impl<T> ReadMessage for Option<T>
where
    T: ReadMessage,
{
    fn read_string(self) -> Result<String, DecodeError> {
        self.map(ReadMessage::read_string)
            .transpose()?
            .ok_or_else(|| DecodeError::Eof)
    }
}

impl<T, E> ReadMessage for Result<T, E>
where
    T: ReadMessage,
    E: Into<DecodeError>,
{
    fn read_string(self) -> Result<String, DecodeError> {
        self.map_err(Into::into).and_then(ReadMessage::read_string)
    }
}

/// A `Decoder` provides `read_message()` from a [`futures::Stream`]
pub struct StreamDecoder<IO> {
    stream: IO,
    buf: String,
    pos: usize,
    back_queue: VecDeque<IrcMessage<'static>>,
}

impl<IO, I> StreamDecoder<IO>
where
    IO: Stream<Item = I> + Unpin,
    <IO as Stream>::Item: ReadMessage + Send + Sync,
{
    /// Create a `StreamDecoder` from a [`futures::Stream`].
    pub fn new(stream: IO) -> Self {
        Self {
            stream,
            buf: String::new(),
            pos: 0,
            back_queue: VecDeque::new(),
        }
    }

    /// Read the next message.
    ///
    /// This returns a owned [`IrcMessage`].
    pub async fn read_message(&mut self) -> Result<IrcMessage<'static>, DecodeError> {
        if let Some(msg) = self.back_queue.pop_front() {
            return Ok(msg);
        }
        if self.pos != 0 {
            return self.parse_message();
        }

        self.buf.clear();

        match self.stream.next().await {
            Some(res) => {
                let data = res.read_string()?;
                self.buf.extend(Some(data));
                self.parse_message()
            }
            None => Err(DecodeError::Eof),
        }
    }

    // TODO this should respond to PINGs
    /// Wait for a specific event.
    ///
    /// This returns the specific matched event and any missed messages read before this returns.
    ///
    /// You can use [Decoder::extend][extend] to feed these messages back into the decoder.
    ///
    /// [extend]: StreamDecoder::extend()
    pub async fn wait_for(
        &mut self,
        event: Event,
    ) -> Result<(IrcMessage<'static>, Vec<IrcMessage<'static>>), DecodeError> {
        let mut missed = vec![];
        loop {
            let msg = self.read_message().await?;
            if Event::from_raw(msg.get_command()) == event {
                break Ok((msg.into_owned(), missed));
            } else {
                missed.push(msg.into_owned())
            }
        }
    }

    fn parse_message(&mut self) -> Result<IrcMessage<'static>, DecodeError> {
        match self.buf[self.pos..].as_bytes() {
            [.., b'\r', b'\n'] => {}
            [.., b'\n'] => {
                self.buf.pop();
                self.buf.push_str("\r\n");
            }
            [..] => self.buf.push_str("\r\n"),
        };

        let (p, msg) = crate::irc::parse_one(&self.buf[self.pos..]) //
            .map_err(DecodeError::ParseError)?;

        self.pos = if p > 0 { self.pos + p } else { 0 };

        Ok(msg.into_owned())
    }
}

impl<IO> Extend<IrcMessage<'static>> for StreamDecoder<IO> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = IrcMessage<'static>>,
    {
        self.back_queue.extend(iter)
    }
}

impl<IO> std::fmt::Debug for StreamDecoder<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamDecoder").finish()
    }
}

impl<IO, I> Stream for StreamDecoder<IO>
where
    IO: Stream<Item = I> + Unpin,
    <IO as Stream>::Item: ReadMessage + Send + Sync,
{
    type Item = Result<IrcMessage<'static>, DecodeError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use std::future::Future as _;

        let this = self.get_mut();
        let fut = this.read_message();
        futures_lite::pin!(fut);

        let res = match futures_lite::ready!(fut.poll(cx)) {
            Err(DecodeError::Eof) => None,
            Err(DecodeError::Io(err)) if crate::util::is_blocking_error(&err) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Ok(msg) => Some(Ok(msg.into_owned())),
            Err(err) => Some(Err(err)),
        };

        Poll::Ready(res)
    }
}
