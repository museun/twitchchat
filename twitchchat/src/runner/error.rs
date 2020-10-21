use crate::{DecodeError, MessageError};

/// An error returned by helpers in the [`runner`][runner] module.
///
/// [runner]: crate::runner
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured
    Io(std::io::Error),
    /// Invalid utf-8 was parsed (either you sent invalid utf-8, or Twitch did and we read it).
    InvalidUtf8(std::str::Utf8Error),
    /// We could not parse a message -- this should never happen
    ParsingFailure(MessageError),
    /// You requested a capability and Twitch rejected it
    InvalidCap {
        /// The capability name
        cap: String,
    },
    /// You provided an invalid OAuth token
    BadPass,
    /// Your connection timed out.
    TimedOut,
    /// Twitch restarted the server, you should reconnect.
    ShouldReconnect,
    /// An unexpected EOF was found -- this means the connectionc losed abnormally.
    UnexpectedEof,
    /// An EOF was read -- this isn't a real error
    Eof,

    #[cfg(feature = "ws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    /// A Soketto handshake error (websockets)
    Handshake(soketto::handshake::Error),

    #[cfg(feature = "ws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    /// A Soketto connection error (websockets)
    Connection(soketto::connection::Error),

    #[cfg(feature = "ws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    /// A Soketto error, with status code (websockets)
    CannotConnect {
        /// The status code
        status_code: u16,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::InvalidUtf8(err) => write!(f, "invalid utf-8 while parsing: {}", err),
            Self::ParsingFailure(err) => write!(f, "could not parse message: {}", err),
            Self::InvalidCap { cap } => {
                write!(f, "request capability '{}' was not acknowledged", cap)
            }
            Self::BadPass => f.write_str("an invalid oauth token was provided"),
            Self::TimedOut => write!(f, "your connection timed out"),
            Self::ShouldReconnect => write!(f, "you should reconnect. Twitch restarted the server"),
            Self::UnexpectedEof => write!(f, "reached an unexpected EOF"),

            Error::Eof => f.write_str("reach an expected EOF"),

            #[cfg(feature = "ws")]
            Error::Handshake(err) => write!(f, "soketto handshake error: {}", err),

            #[cfg(feature = "ws")]
            Error::Connection(err) => write!(f, "soketto connection error: {}", err),

            #[cfg(feature = "ws")]
            Error::CannotConnect { status_code } => {
                write!(f, "soketto error, status code: {}", status_code)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::InvalidUtf8(err) => Some(err),
            Self::ParsingFailure(err) => Some(err),
            #[cfg(feature = "ws")]
            Self::Handshake(err) => Some(err),
            #[cfg(feature = "ws")]
            Self::Connection(err) => Some(err),
            _ => None,
        }
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        match err {
            DecodeError::Io(err) => Self::Io(err),
            DecodeError::InvalidUtf8(err) => Self::InvalidUtf8(err),
            DecodeError::ParseError(err) => Self::ParsingFailure(err),
            DecodeError::Eof => Self::UnexpectedEof,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<MessageError> for Error {
    fn from(err: MessageError) -> Self {
        Self::ParsingFailure(err)
    }
}

cfg_ws! {
impl From<soketto::handshake::Error> for Error {
    fn from(err: soketto::handshake::Error) -> Self {
        Self::Handshake(err)
    }
}
}

cfg_ws! {
impl From<soketto::connection::Error> for Error {
    fn from(err: soketto::connection::Error) -> Self {
        Self::Connection(err)
    }
}
}
