//! Helpers for building event loops
//!

mod capabilities;
pub use capabilities::Capabilities;

mod identity;
pub use identity::Identity;

mod error;
pub use error::Error;

mod timeout;
pub use timeout::{
    idle_detection_loop_sync, Activity, ActivityReceiver, ActivitySender, TIMEOUT, WINDOW,
};
cfg_async! { pub use timeout::idle_detection_loop; }

cfg_async! { #[cfg(feature = "writer")] pub use timeout::respond_to_idle_events; }

mod wait_for_ready;
pub use wait_for_ready::wait_until_ready_sync;
cfg_async! { pub use wait_for_ready::wait_until_ready; }

mod wait_for;
pub use wait_for::{wait_for_sync, Event};
cfg_async! { pub use wait_for::wait_for; }
