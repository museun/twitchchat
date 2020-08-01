pub mod decoder;
pub use decoder::{Decoder, DecoderAsync, Error as DecodeError};

mod dispatcher;
pub use dispatcher::{DispatchError, Dispatcher};

mod encoder;
pub use encoder::{AsyncEncoder, Encodable, Encoder};

pub mod commands;
pub mod messages;

mod from_irc_message;
pub use from_irc_message::{FromIrcMessage, InvalidMessage};

pub mod irc;
use irc::{IrcMessage, TagIndices, Tags};

mod maybe_owned;
pub use maybe_owned::{MaybeOwned as Str, MaybeOwnedIndex as StrIndex};

pub mod validator;
use validator::Validator;

// a public dep
pub use simple_event_map::{EventMap, EventStream};

#[cfg(feature = "serde")]
mod serde;

#[cfg(test)]
mod commands_test;
