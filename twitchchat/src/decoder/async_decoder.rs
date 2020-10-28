use super::DecodeError;
use crate::{
    irc::IrcMessage,
    wait_for::{wait_inner, Event, State},
    IntoOwned as _,
};

use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{
    io::BufReader as AsyncBufReader, AsyncBufReadExt, AsyncRead, AsyncWrite, Stream,
};

/// A decoder over [`futures::AsyncRead`] that produces [`IrcMessage`]s
///
/// This will return an [`DecodeError::Eof`] when its done reading manually.
///
/// When reading it as a stream, `Eof` will signal the end of the stream (e.g. `None`)
pub struct AsyncDecoder<R> {
    reader: AsyncBufReader<R>,
    buf: Vec<u8>,
    back_queue: VecDeque<IrcMessage<'static>>,
}

impl<R> AsyncDecoder<R>
where
    R: AsyncRead + Send + Unpin,
{
    /// Create a new [`AsyncDecoder`] from this [`futures::AsyncRead`] instance
    pub fn new(reader: R) -> Self {
        Self {
            reader: AsyncBufReader::new(reader),
            buf: Vec::with_capacity(1024),
            back_queue: VecDeque::new(),
        }
    }

    /// Read the next message.
    ///
    /// This returns a borrowed [`IrcMessage`] which is valid until the next [`AsyncDecoder`] call is made.
    ///
    /// If you just want an owned one, use the [`AsyncDecoder`] as an stream. e.g. dec.next().
    pub async fn read_message(&mut self) -> Result<IrcMessage<'_>, DecodeError> {
        if let Some(msg) = self.back_queue.pop_front() {
            return Ok(msg);
        }

        self.buf.clear();
        let n = self
            .reader
            .read_until(b'\n', &mut self.buf)
            .await
            .map_err(DecodeError::Io)?;
        if n == 0 {
            return Err(DecodeError::Eof);
        }

        let str = std::str::from_utf8(&self.buf[..n]).map_err(DecodeError::InvalidUtf8)?;
        log::trace!("< {}", str.escape_debug());

        // this should only ever parse 1 message
        crate::irc::parse_one(str)
            .map_err(DecodeError::ParseError)
            .map(|(_, msg)| msg)
    }

    /// Wait for a specific event.
    ///
    /// This returns the specific matched event and any missed messages read before this returns.
    ///
    /// You can use [Decoder::extend][extend] to feed these messages back into the decoder.
    ///
    /// [extend]: AsyncDecoder::extend()
    pub async fn wait_for(
        &mut self,
        event: Event,
    ) -> Result<(IrcMessage<'static>, Vec<IrcMessage<'static>>), DecodeError>
    where
        R: AsyncWrite,
    {
        let mut missed = vec![];
        loop {
            match wait_inner(self.read_message().await, event)? {
                State::Done(msg) => break Ok((msg.into_owned(), missed)),
                State::Requeue(msg) => missed.push(msg.into_owned()),
                State::Yield => futures_lite::future::yield_now().await,
            }
        }
    }

    /// Consume the decoder returning the inner Reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

impl<R> Extend<IrcMessage<'static>> for AsyncDecoder<R> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = IrcMessage<'static>>,
    {
        self.back_queue.extend(iter)
    }
}

impl<R> std::fmt::Debug for AsyncDecoder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncDecoder").finish()
    }
}

/// This will produce `Result<IrcMessage<'static>, DecodeError>` until an `Eof` is received
impl<R> Stream for AsyncDecoder<R>
where
    R: AsyncRead + Send + Unpin,
{
    type Item = Result<IrcMessage<'static>, DecodeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_async() {
        use futures_lite::stream::StreamExt as _;
        let fut = async move {
            let data = b"hello\r\nworld\r\ntesting this\r\nand another thing\r\n".to_vec();
            let mut reader = futures_lite::io::Cursor::new(data);

            // reading from the stream won't produce the EOF
            let out = AsyncDecoder::new(&mut reader).collect::<Vec<_>>().await;
            // you cannot collect a Stream into aa result. so lets just do it manually
            let out = out
                .into_iter()
                .collect::<Result<Vec<_>, DecodeError>>()
                .unwrap();
            assert_eq!(out.len(), 4);

            reader.set_position(0);

            // manually reading should produce an EOF
            let mut dec = AsyncDecoder::new(reader);
            for _ in 0..4 {
                dec.read_message().await.unwrap();
            }
            assert!(matches!(
                dec.read_message().await.unwrap_err(),
                DecodeError::Eof
            ))
        };

        futures_lite::future::block_on(fut);
    }
}
