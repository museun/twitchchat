use crate::{DecodeError, DispatchError};

/// An error produce by a Runner
#[derive(Debug)]
pub enum Error {
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
