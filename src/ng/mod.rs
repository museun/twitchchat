#![cfg_attr(debug_assertions, allow(dead_code,))]

// dispatcher
// runner
// control
// eventstream

mod maybe_owned;
pub use maybe_owned::{MaybeOwned as Str, Reborrow};

mod dispatcher;
pub use dispatcher::{DispatchError, Dispatcher};

mod event_map;
pub use event_map::EventMap;

mod event_stream;
pub use event_stream::EventStream;

mod encoder;
pub use encoder::{AsyncEncoder, Encodable, Encoder};

pub mod commands;
pub mod messages;

pub mod channel;
pub use channel::{Receiver, Sender};

mod irc;
pub use irc::{IrcMessage, Prefix, Tags};
