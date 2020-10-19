// cfg_async! {
use crate::{irc::IrcMessage, DecodeError, IntoOwned};

use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{io::BufReader as AsyncBufReader, AsyncBufReadExt, AsyncRead, Stream};

/// A decoder over [`futures_io::AsyncRead`][write] that produces [`IrcMessage`]s
///
/// This will return an [`DecodeError::Eof`] when its done reading manually.
///
/// When reading it as a stream, `Eof` will signal the end of the stream (e.g. `None`)
///
/// [write]: https://docs.rs/futures-io/0.3.6/futures_io/trait.AsyncWrite.html
pub struct AsyncDecoder<R> {
    reader: AsyncBufReader<R>,
    buf: Vec<u8>,
    back_queue: VecDeque<IrcMessage<'static>>,
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

impl<R: AsyncRead + Send + Sync + Unpin> AsyncDecoder<R> {
    /// Create a new [`AsyncDecoder`] from this [`futures_io::AsyncRead`][read] instance
    ///
    /// [read]: https://docs.rs/futures-io/0.3.6/futures_io/trait.AsyncRead.html
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

    /// Consume the decoder returning the inner Reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

/// This will produce `Result<IrcMessage<'static>, DecodeError>` until an `Eof` is received
impl<R> Stream for AsyncDecoder<R>
where
    R: AsyncRead + Send + Sync + Unpin,
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
// }
