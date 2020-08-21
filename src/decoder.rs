//! Decoding utilities.
//!
//! A decoder lets you decode messages from an `std::io::Read` (or `futures::io::AsyncRead` for async) in either an iterative fashion, or one-by-one.
//!
//! When not using the Iterator (or Stream), you'll get a borrowed message from the reader that is valid until the next read.
//!
//! With the Iterator (or Stream) interface, it'll return an owned messages.
//!
//! This crate provides both 'Sync' (Iterator based) and 'Async' (Stream based) decoding.
//! * sync: [`Decoder`][decoder]
//! * async: [`AsyncDecoder`][async_decoder]
//!
//! # Borrowed messages
//! ```
//! let input = "@key1=val;key2=true :user!user@user PRIVMSG #some_channel :\x01ACTION hello world\x01\r\n";
//! let mut reader = std::io::Cursor::new(input.as_bytes());
//!
//! // you can either &mut borrow the reader, or let the Decoder take ownership.
//! // ff it takes ownership you can retrieve the inner reader later.
//! let mut decoder = twitchchat::Decoder::new(&mut reader);
//!
//! // returns whether the message was valid
//! // this'll block until it can read a 'full' message (e.g. one delimited by `\r\n`).
//! let msg = decoder.read_message().unwrap();
//!
//! // msg is borrowed until the next `read_message()`
//! // you can turn a borrowed message into an owned message by using the twitchchat::IntoOwned trait.
//! use twitchchat::IntoOwned as _;
//! let owned: twitchchat::IrcMessage<'static> = msg.into_owned();
//! ```
//!
//! # Owned messages
//! ```
//! let input = "@key1=val;key2=true :user!user@user PRIVMSG #some_channel :\x01ACTION hello world\x01\r\n";
//! let mut reader = std::io::Cursor::new(input.as_bytes());
//!
//! // you can either &mut borrow the reader, or let the Decoder take ownership.
//! // ff it takes ownership you can retrieve the inner reader later.
//! for msg in twitchchat::Decoder::new(&mut reader) {
//!     // this yields whether the message was valid or not
//!     // this'll block until it can read a 'full' message (e.g. one delimited by `\r\n`).
//!
//!     // notice its already owned here (denoted by the 'static lifetime)
//!     let msg: twitchchat::IrcMessage<'static> = msg.unwrap();
//! }
//! ```
//! [decoder]: struct.Decoder.html
//! [async_decoder]: struct.AsyncDecoder.html

use crate::{irc::IrcMessage, IntoOwned, MessageError};

use std::{
    future::Future,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{io::BufReader as AsyncBufReader, AsyncBufReadExt, AsyncRead, Stream};

/// An error produced by a Decoder.
#[derive(Debug)]
pub enum DecodeError {
    /// An I/O error occurred
    Io(std::io::Error),
    /// Invalid UTf-8 was read.
    InvalidUtf8(std::str::Utf8Error),
    /// Could not parse the IRC message
    ParseError(MessageError),
    /// EOF was reached
    Eof,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::InvalidUtf8(err) => write!(f, "invalid utf8: {}", err),
            Self::ParseError(err) => write!(f, "parse error: {}", err),
            Self::Eof => f.write_str("end of file reached"),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::InvalidUtf8(err) => Some(err),
            Self::ParseError(err) => Some(err),
            _ => None,
        }
    }
}

/// A decoder over `std::io::Read` that produces `IrcMessage`s
///
/// This will return an `DecodeError::Eof` when reading manually.
///
/// When reading it as a iterator, `Eof` will signal the end of the iterator (e.g. `None`)
pub struct Decoder<R> {
    reader: std::io::BufReader<R>,
    buf: Vec<u8>,
}

impl<R> std::fmt::Debug for Decoder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

impl<R: Read> Decoder<R> {
    /// Create a new Decoder from this `std::io::Read` instance
    pub fn new(reader: R) -> Self {
        Self {
            reader: std::io::BufReader::new(reader),
            buf: Vec::with_capacity(1024),
        }
    }

    /// Read the next message.
    ///
    /// This returns a borrowed IrcMessage which is valid until the next Decoder call is made.
    ///
    /// If you just want an owned one, use the Decoder as an iterator. e.g. dec.next().
    pub fn read_message(&mut self) -> Result<IrcMessage<'_>, DecodeError> {
        use std::io::BufRead;

        self.buf.clear();
        let n = self
            .reader
            .read_until(b'\n', &mut self.buf)
            .map_err(DecodeError::Io)?;
        if n == 0 {
            return Err(DecodeError::Eof);
        }

        let str = std::str::from_utf8(&self.buf[..n]).map_err(DecodeError::InvalidUtf8)?;

        // this should only ever parse 1 message
        crate::irc::parse_one(str)
            .map_err(DecodeError::ParseError)
            .map(|(_, msg)| msg)
    }

    /// Returns an iterator over messages.
    ///
    /// This will produce Results of Messages until an EOF is received
    pub fn iter(&mut self) -> &mut Self {
        self
    }

    /// Consume the decoder returning the inner Reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

/// This will produce `Result<IrcMessage<'static>, DecodeError>` until an `Eof` is received
impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<IrcMessage<'static>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_message() {
            Err(DecodeError::Eof) => None,
            Ok(msg) => Some(Ok(msg.into_owned())),
            Err(err) => Some(Err(err)),
        }
    }
}

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
    fn read_sync() {
        let data = b"hello\r\nworld\r\ntesting this\r\nand another thing\r\n".to_vec();
        let mut reader = std::io::Cursor::new(data);

        // reading from the iterator won't produce the EOF
        let v = Decoder::new(&mut reader)
            .iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        // no EOF
        assert_eq!(v.len(), 4);

        reader.set_position(0);
        // manually reading should produce an EOF
        let mut dec = Decoder::new(reader);
        for _ in 0..4 {
            dec.read_message().unwrap();
        }
        assert!(matches!(dec.read_message().unwrap_err(), DecodeError::Eof))
    }

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
