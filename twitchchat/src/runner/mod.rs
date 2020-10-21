//! Helpers for building event loops
//!

mod capabilities;
mod error;
mod identity;
mod timeout;
mod wait_for;
pub(crate) mod wait_for_ready;

pub use capabilities::Capabilities;
pub use error::Error;
pub use identity::Identity;

pub use timeout::{idle_detection_loop_sync, TIMEOUT, WINDOW};
pub use timeout::{Activity, ActivityReceiver, ActivitySender};

pub use wait_for::{wait_for_sync, Event};
pub use wait_for_ready::wait_until_ready_sync;

cfg_async! {
    pub use timeout::idle_detection_loop;
    pub use wait_for_ready::wait_until_ready;

    cfg_writer! {
        pub use timeout::respond_to_idle_events;
    }

    pub use wait_for::wait_for;
}
