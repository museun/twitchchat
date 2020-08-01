// this has to be first for the macro
#[cfg(feature = "serde")]
mod serde;

pub mod decoder;
pub use decoder::{Decoder, DecoderAsync, Error as DecodeError};

mod dispatcher;
pub use dispatcher::{DispatchError, Dispatcher};

// a public dep
pub use simple_event_map::{EventMap, EventStream};

mod encoder;
pub use encoder::{AsyncEncoder, Encodable, Encoder};

pub mod commands;
pub mod messages;

// do we want to expose this?
mod channel;

mod from_irc_message;
pub use from_irc_message::{FromIrcMessage, InvalidMessage};

pub mod irc;
use irc::{IrcMessage, TagIndices, Tags};

mod maybe_owned;
pub use maybe_owned::{MaybeOwned as Str, MaybeOwnedIndex as StrIndex};

pub mod validator;
// TODO hide this ?
use validator::Validator;

#[cfg(test)]
mod commands_test;
