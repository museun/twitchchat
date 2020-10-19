use crate::MessageError;

/// An error produced by a Decoder.
#[derive(Debug)]
#[non_exhaustive]
pub enum DecodeError {
    /// An I/O error occurred
    Io(std::io::Error),
    /// Invalid UTF-8 was read.
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
