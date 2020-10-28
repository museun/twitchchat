use crate::{decoder::DecodeError, wait_for::CheckError};

/// An error returned by one of the various `wait_until_ready()` methods
#[derive(Debug)]
#[non_exhaustive]
pub enum HandshakeError {
    /// An invalid OAuth token was provided
    BadPass,
    /// The server restarted, you should reconnect
    ShouldReconnect,
    /// You provided an invalid capability
    InvalidCapability(String),
    /// Could not encode the required messages to the writer
    Encode(std::io::Error),
    /// Could not decode messages from the reader
    Decode(DecodeError),
}

impl HandshakeError {
    pub(super) fn from_check_err(err: CheckError) -> Self {
        match err {
            CheckError::InvalidCap(cap) => Self::InvalidCapability(cap),
            CheckError::BadPass => Self::BadPass,
            CheckError::ShouldReconnect => Self::ShouldReconnect,
        }
    }
}

impl From<std::io::Error> for HandshakeError {
    fn from(err: std::io::Error) -> Self {
        Self::Encode(err)
    }
}

impl From<DecodeError> for HandshakeError {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadPass => f.write_str("an invalid oauth token was provided"),

            Self::ShouldReconnect => {
                f.write_str("you should reconnect. Twitch restarted the server")
            }

            Self::InvalidCapability(cap) => write!(
                f,
                "an invalid capability was provided '{}'",
                cap.escape_debug()
            ),

            Self::Encode(err) => err.fmt(f),
            Self::Decode(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for HandshakeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(err) => Some(err),
            Self::Encode(err) => Some(err),
            _ => None,
        }
    }
}
