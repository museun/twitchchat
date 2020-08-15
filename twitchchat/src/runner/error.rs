use crate::{DecodeError, DispatchError};

/// An error produce by a Runner
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// You were waiting for a message Twitch would never send.
    ///
    /// You should make sure to send the right capabilities for that message
    RequiredCaps(crate::Capability, &'static str),
    /// There was an error while waiting for a message
    WaitForMessage(&'static str),
    /// Invalid connection
    InvalidConnection,
    /// There was a dispatch error    
    Dispatch(DispatchError),
    /// There was a decode error
    Decode(DecodeError),
    /// There was an i/o error
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequiredCaps(cap, ty) => write!(
                f,
                "you did not send the right Capability ({:?}) for '{}' to be received",
                cap, ty
            ),
            Self::WaitForMessage(ty) => write!(
                f,
                "connection closed before message of '{}' was received",
                ty
            ),
            Self::InvalidConnection => write!(f, "twitch never gave us your username"),
            Self::Dispatch(err) => write!(f, "dispatching error: {}", err),
            Self::Decode(err) => write!(f, "decoding error: {}", err),
            Self::Io(err) => write!(f, "i/o error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch(err) => Some(err),
            Self::Decode(err) => Some(err),
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<DispatchError> for Error {
    fn from(err: DispatchError) -> Self {
        Self::Dispatch(err)
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
