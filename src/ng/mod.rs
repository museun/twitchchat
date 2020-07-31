// this has to be first for the macro
#[macro_use]
mod serde;

// mod dispatcher;
// pub use dispatcher::{DispatchError, Dispatcher};

mod event_map;
pub use event_map::EventMap;

mod event_stream;
pub use event_stream::EventStream;

mod encoder;
pub use encoder::{AsyncEncoder, Encodable, Encoder};

// pub mod commands;
pub mod messages;

pub mod channel;
pub use channel::{Receiver, Sender};

mod from_irc_message;
pub use from_irc_message::{FromIrcMessage, InvalidMessage};

pub mod irc;
use irc::{IrcMessage, TagIndices, Tags};

mod maybe_owned;
pub use maybe_owned::{MaybeOwned as Str, MaybeOwnedIndex as StrIndex};

pub mod validator;
// TODO hide this ?
use validator::Validator;
