//! A set of runners for managing a `event loop`
//!
//! To use the [`AsyncRunner`][async_runner]:
//! 1. choose [`Connector`][connector] from [`connectors`][connectors].
//! 1. create a [`UserConfig`][user_config].
//! 1. create and connect the [`AsyncRunner`][async_runner] via its [`AsyncRunner::connect()`][connect] method
//! 1. now you're connected to Twitch, so next things you can do.
//!     1. join a channel with: [`AsyncRunner::join()`][join],
//!     1. write messages with the [`AsyncWriter`][async_writer] provided by [`AsyncRunner::writer()`][writer].
//!     1. signal you want to quit with the [`AsyncRunner::quit_handle()`][quit]
//!
//! [async_runner]: struct.AsyncRunner.html
//! [connector]: ../connector/trait.Connector.html
//! [connectors]: ../connector/index.html
//! [user_config]: ../twitch/struct.UserConfig.html
//! [connect]: struct.AsyncRunner.html#method.connect
//! [join]: struct.AsyncRunner.html#method.join
//! [async_writer]: ../writer/struct.AsyncWriter.html
//! [writer]: struct.AsyncRunner.html#method.writer
//! [quit]: struct.AsyncRunner.html#method.quit_handle

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
