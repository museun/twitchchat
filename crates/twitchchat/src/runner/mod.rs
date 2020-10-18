//! A set of runners for managing a `event loop`
//!
//! To use the [`AsyncRunner`]:
//! 1. create a [`UserConfig`](crate::UserConfig).
//! 1. create and connect the [`AsyncRunner`] via its [`AsyncRunner::connect()`] method
//!     1. this takes a [`futures_io::AsyncRead`][async_read] + [`futures_io::AsyncWrite`][async_write]
//! 1. now you're connected to Twitch, so next things you can do.
//!     1. join a channel with: [`AsyncRunner::join()`],
//!     1. write messages with the [`AsyncWriter`](crate::writer::AsyncWriter) provided by [`AsyncRunner::writer()`].
//!     1. signal you want to quit with the [`AsyncRunner::quit_handle()`]
//!
//! [async_read]: https://docs.rs/futures-io/0.3.6/futures_io/trait.AsyncRead.html
//! [async_write]: https://docs.rs/futures-io/0.3.6/futures_io/trait.AsyncWrite.html

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
    mod rate_limit;
}

cfg_async! {
    mod channel;
    pub use channel::Channel;
}

cfg_async! {
    mod async_runner;
    pub use async_runner::AsyncRunner;
}

cfg_async! {
    #[doc(inline)]
    pub use crate::util::NotifyHandle;
}
