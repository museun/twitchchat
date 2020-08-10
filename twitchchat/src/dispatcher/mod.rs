use crate::InvalidMessage;
use std::convert::Infallible;

mod sync_dispatcher;
pub use sync_dispatcher::SyncDispatcher;

mod async_dispatcher;
pub use async_dispatcher::AsyncDispatcher;

/// An error produced by the Dispatcher
#[derive(Debug)]
#[non_exhaustive]
pub enum DispatchError {
    /// The message type was wrong -- this will only happen on user-defined events.
    InvalidMessage(InvalidMessage),
    /// A custom error message -- this will only happen on user-defined events.
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

impl DispatchError {
    /// Create a new custom error message type
    pub fn custom(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Custom(Box::new(err))
    }
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMessage(err) => write!(f, "invalid message: {}", err),
            Self::Custom(err) => write!(f, "unknown error: {}", err),
        }
    }
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidMessage(err) => Some(err),
            Self::Custom(err) => Some(&**err),
        }
    }
}

impl From<InvalidMessage> for DispatchError {
    fn from(msg: InvalidMessage) -> Self {
        Self::InvalidMessage(msg)
    }
}

impl From<Infallible> for DispatchError {
    fn from(_: Infallible) -> Self {
        unreachable!("you cannot produce this error")
    }
}
