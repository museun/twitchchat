use crate::{DecodeError, DispatchError};

#[derive(Debug)]
pub enum RunnerError {
    Dispatch(DispatchError),
    Decode(DecodeError),
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
