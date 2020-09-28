use crate::{IntoOwned as _, IrcMessage, MessageError};
use std::io::{BufRead, BufReader, Read};

/// An error produced by a Decoder.
#[derive(Debug)]
#[non_exhaustive]
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

/// A decoder over [std::io::Read] that produces [IrcMessage]s
///
/// This will return an [DecodeError::Eof] when reading manually.
///
/// When reading it as a iterator, `Eof` will signal the end of the iterator (e.g. `None`)
pub struct Decoder<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
}

impl<R> std::fmt::Debug for Decoder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

impl<R> Decoder<R>
where
    R: Read,
{
    /// Create a new Decoder from this [std::io::Read] instance
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            buf: Vec::with_capacity(1024),
        }
    }

    /// Read the next message.
    ///
    /// This returns a borrowed [IrcMessage] which is valid until the next Decoder call is made.
    ///
    /// If you just want an owned one, use the [Decoder] as an iterator. e.g. dec.next().
    pub fn read_message(&mut self) -> Result<IrcMessage<'_>, DecodeError> {
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
}
