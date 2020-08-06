use crate::{irc::IrcMessage, IntoOwned, InvalidMessage};

use std::{
    future::Future,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{io::BufReader as AsyncBufReader, AsyncBufReadExt, AsyncRead, Stream};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidUtf8(std::str::Utf8Error),
    ParseError(InvalidMessage),
    Eof,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::InvalidUtf8(err) => write!(f, "invalid utf8: {}", err),
            Self::ParseError(err) => write!(f, "parse error: {}", err),
            Self::Eof => f.write_str("end of file reached"),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::InvalidUtf8(err) => Some(err),
            Self::ParseError(err) => Some(err),
            _ => None,
        }
    }
}

pub struct Decoder<R> {
    reader: std::io::BufReader<R>,
    buf: Vec<u8>,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: std::io::BufReader::new(reader),
            buf: Vec::with_capacity(1024),
        }
    }

    pub fn read_message(&mut self) -> Result<IrcMessage<'_>, Error> {
        use std::io::BufRead;

        self.buf.clear();
        let n = self
            .reader
            .read_until(b'\n', &mut self.buf)
            .map_err(Error::Io)?;
        if n == 0 {
            return Err(Error::Eof);
        }

        let str = std::str::from_utf8(&self.buf[..n]).map_err(Error::InvalidUtf8)?;

        // this should only ever parse 1 message
        crate::irc::parse_one(str)
            .map_err(Error::ParseError)
            .map(|(_, msg)| msg)
    }

    pub fn iter(&mut self) -> &mut Self {
        self
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = IrcMessage<'static>;
    fn next(&mut self) -> Option<Self::Item> {
        self.read_message().ok().map(IrcMessage::into_owned)
    }
}

pub struct DecoderAsync<R> {
    reader: AsyncBufReader<R>,
    buf: Vec<u8>,
}

impl<R: AsyncRead + Unpin> DecoderAsync<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: AsyncBufReader::new(reader),
            buf: Vec::with_capacity(1024),
        }
    }

    pub async fn read_message(&mut self) -> Result<IrcMessage<'_>, Error> {
        self.buf.clear();
        let n = self
            .reader
            .read_until(b'\n', &mut self.buf)
            .await
            .map_err(Error::Io)?;
        if n == 0 {
            return Err(Error::Eof);
        }

        let str = std::str::from_utf8(&self.buf[..n]).map_err(Error::InvalidUtf8)?;

        // this should only ever parse 1 message
        crate::irc::parse_one(str)
            .map_err(Error::ParseError)
            .map(|(_, msg)| msg)
    }
}

impl<R: AsyncRead + Unpin> Stream for DecoderAsync<R> {
    type Item = IrcMessage<'static>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut();
        let fut = this.read_message();
        futures_lite::pin!(fut);
        fut.poll(cx).map(|s| s.ok().map(IntoOwned::into_owned))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_sync() {
        let data = b"hello\r\nworld\r\ntesting this\r\nand another thing\r\n".to_vec();
        let reader = std::io::Cursor::new(data);

        let v = Decoder::new(reader).iter().collect::<Vec<_>>();
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn read_async() {
        use futures_lite::stream::StreamExt as _;
        let fut = async move {
            let data = b"hello\r\nworld\r\ntesting this\r\nand another thing\r\n".to_vec();
            let reader = futures_lite::io::Cursor::new(data);

            let out = DecoderAsync::new(reader).collect::<Vec<_>>().await;
            assert_eq!(out.len(), 4);
        };

        async_executor::LocalExecutor::new().run(fut);
    }
}
