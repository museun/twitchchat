use crate::{DecodeError, InvalidMessage};

/// An error Returned by a Runner
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured
    Io(std::io::Error),
    /// Invalid utf-8 was parsed (either you sent invalid utf-8, or twitch did and we read it).
    InvalidUtf8(std::str::Utf8Error),
    /// We could not parse a message -- this should never happen
    ParsingFailure(InvalidMessage),
    /// You requested a capability and Twitch rejected it
    InvalidCap {
        /// The capability name
        cap: String,
    },
    /// You're already on that channel
    AlreadyOnChannel {
        /// The channel name
        channel: String,
    },
    /// You weren't on that channel
    NotOnChannel {
        /// The channel name
        channel: String,
    },
    /// Your connection timed out.
    TimedOut,
    /// Twitch restarted the server, you should reconnect.
    ShouldReconnect,
    /// An unexpected EOF was found -- this means the connectionc losed abnormally.
    UnexpectedEof,
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
            Self::AlreadyOnChannel { channel } => write!(f, "already on channel '{}'", channel),
            Self::NotOnChannel { channel } => write!(f, "not on channel '{}'", channel),
            Self::TimedOut => write!(f, "your connection timed out"),
            Self::ShouldReconnect => write!(f, "you should reconnect. Twitch restarted the server"),
            Self::UnexpectedEof => write!(f, "reached an unexpected EOF"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::InvalidUtf8(err) => Some(err),
            Self::ParsingFailure(err) => Some(err),
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
