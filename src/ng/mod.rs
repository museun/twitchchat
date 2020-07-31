// this has to be first for the macro
#[cfg(feature = "serde")]
mod serde;

macro_rules! serde_struct {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_struct!(@one $e)),*]); };

    ($ty:ident { $($field:ident),* $(,)? }) => {
        #[cfg(feature = "serde")]
        impl<'t> ::serde::Serialize for $ty<'t> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeStruct as _;
                let len = serde_struct!(@len $($field),*);
                let mut s = serializer.serialize_struct(stringify!($ty), len)?;
                $( s.serialize_field(stringify!($field), &self.$field())?; )*
                s.end()
            }
        }

        serde_struct!{ @de $ty }
    };

    (@de $ty:ident) => {
        #[cfg(feature = "serde")]
        impl<'de, 't> ::serde::Deserialize<'de> for $ty<'t> {
            fn deserialize<D>(deserializer: D) -> Result<$ty<'t>, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                deserializer.deserialize_map($crate::ng::serde::RawVisitor::default())
            }
        }
    };
}

mod dispatcher;
pub use dispatcher::{DispatchError, Dispatcher};

mod event_map;
pub use event_map::{EventMap, Senders};

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
