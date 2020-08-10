use crate::{DecodeError, DispatchError};

/// An error produce by the runner
#[derive(Debug)]
pub enum RunnerError {
    /// There was a dispatch error
    Dispatch(DispatchError),
    /// There was a decode error
    Decode(DecodeError),
    /// There was an i/o error
    Io(std::io::Error),
}

impl From<DispatchError> for RunnerError {
    fn from(err: DispatchError) -> Self {
        Self::Dispatch(err)
    }
}

impl From<DecodeError> for RunnerError {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl From<std::io::Error> for RunnerError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
