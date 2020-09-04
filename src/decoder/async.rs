use super::*;
use crate::{irc::IrcMessage, IntoOwned};

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{io::BufReader as AsyncBufReader, AsyncBufReadExt, AsyncRead, Stream};

/// A decoder over `futures::io::AsyncRead` that produces `IrcMessage`s
///
/// This will return an `DecodeError::Eof` when its done reading manually.
///
/// When reading it as a stream, `Eof` will signal the end of the stream (e.g. `None`)
pub struct AsyncDecoder<R> {
    reader: AsyncBufReader<R>,
    buf: Vec<u8>,
}

impl<R> std::fmt::Debug for AsyncDecoder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncDecoder").finish()
    }
}

impl<R: AsyncRead + Send + Sync + Unpin> AsyncDecoder<R> {
    /// Create a new AsyncDecoder from this `futures::io::Read` instance
    pub fn new(reader: R) -> Self {
        Self {
            reader: AsyncBufReader::new(reader),
            buf: Vec::with_capacity(1024),
        }
    }

    /// Read the next message.
    ///
    /// This returns a borrowed IrcMessage which is valid until the next AsyncDecoder call is made.
    ///
    /// If you just want an owned one, use the AsyncDecoder as an stream. e.g. dec.next().
    pub async fn read_message(&mut self) -> Result<IrcMessage<'_>, DecodeError> {
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

        match futures_lite::ready!(fut.poll(cx)) {
            Err(DecodeError::Eof) => Poll::Ready(None),
            Ok(msg) => Poll::Ready(Some(Ok(msg.into_owned()))),
            Err(err) => Poll::Ready(Some(Err(err))),
        }
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
