//! A set of runners for managing a `event loop`
//!
//! To use the [AsyncRunner]:
//! 1. choose [Connector](crate::connector::Connector) from [connectors](crate::connector).
//! 1. create a [UserConfig](crate::UserConfig).
//! 1. create and connect the [AsyncRunner] via its [AsyncRunner::connect()] method
//! 1. now you're connected to Twitch, so next things you can do.
//!     1. join a channel with: [AsyncRunner::join()],
//!     1. write messages with the [AsyncWriter](crate::writer::AsyncWriter) provided by [AsyncRunner::writer()].
//!     1. signal you want to quit with the [AsyncRunner::quit_handle()]
//!

mod status;
pub use status::{Status, StepResult};

mod capabilities;
pub use capabilities::Capabilities;

mod identity;
pub use identity::Identity;

mod error;
pub use error::Error;

#[allow(dead_code)]
mod timeout;

cfg_async! {
    #[doc(inline)]
    pub use crate::util::NotifyHandle;
}
