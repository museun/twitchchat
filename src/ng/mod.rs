#![cfg_attr(debug_assertions, allow(missing_docs, dead_code, unused_imports))]
// this has to be first for the macro
#[macro_use]
mod maybe_owned;
pub use maybe_owned::{AsOwned, MaybeOwned as Str, Reborrow};

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

pub mod irc;
pub use irc::{IrcMessage, Prefix, Tags};
