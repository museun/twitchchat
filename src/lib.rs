#![allow(
    clippy::missing_const_for_fn,
    clippy::redundant_pub_crate,
    clippy::use_self
)]
#![deny(
    deprecated_in_future,
    exported_private_dependencies,
    future_incompatible,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    private_in_public,
    rust_2018_compatibility,
    // rust_2018_idioms, // this complains about elided lifetimes.
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_alias))]
#![cfg_attr(feature = "all_docs", feature(doc_cfg))]
/*!

This crate provides a way to interface with [Twitch](https://dev.twitch.tv/docs/irc)'s chat (via IRC).

Along with the messages as Rust types, it provides methods for sending messages.

---

For twitch types:
* [twitch]
* [messages]
* [commands]
---
For the 'irc' types underneath it all:
* [irc]
---
For an event loop:
* [runner]
---
For just decoding messages:
* [decoder]
---
For just encoding messages:
* [encoder]
---

[runner]: runner/index.html
[encoder]: encoder/index.html
[decoder]: decoder/index.html
[twitch]: twitch/index.html
[messages]: messages/index.html
[commands]: commands/index.html
[irc]: irc/index.html
*/

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "async"))]
            #[cfg_attr(any(feature = "all_docs", docsrs), doc(cfg(feature = "async")))]
            $item
        )*
    };
}

cfg_async! {
    /// A boxed `Future` that is `Send + Sync`
    pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + Sync>>;

    pub mod connector;
    pub mod runner;
    pub mod writer;
    pub mod channel;
}

pub mod rate_limit;

// our internal stuff that should never be exposed
mod ext;

mod util;

/// Prelude with common types
pub mod prelude {
    pub use crate::irc::{IrcMessage, TagIndices, Tags};
    pub use crate::Encodable;
    pub use crate::{commands, messages, twitch};
    pub use crate::{Decoder, Encoder};

    cfg_async! {
        pub use super::decoder::AsyncDecoder;
        pub use super::encoder::AsyncEncoder;
        pub use super::rate_limit::RateClass;
        pub use super::runner::{AsyncRunner, Identity, NotifyHandle, Status};
    }
}

cfg_async! {
    /// An AsyncWriter over an MpscWriter
    pub type Writer = crate::writer::AsyncWriter<crate::writer::MpscWriter>;
}

cfg_async! {
    #[doc(inline)]
    pub use self::decoder::AsyncDecoder;

    // #[doc(inline)]
    // pub use self::encoder::AsyncEncoder;

    pub use self::runner::{AsyncRunner, Status, Error as RunnerError};
}

cfg_async! {
    use crate::channel::Sender;
}

pub use ext::PrivmsgExt;

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
pub const TWITCH_WS_ADDRESS: &str = "ws://irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
pub const TWITCH_WS_ADDRESS_TLS: &str = "wss://irc-ws.chat.twitch.tv:443";

/// A TLS domain for Twitch
pub const TWITCH_TLS_DOMAIN: &str = "irc.chat.twitch.tv";

/// An anonymous login.
pub const ANONYMOUS_LOGIN: (&str, &str) = (JUSTINFAN1234, JUSTINFAN1234);
pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

#[macro_use]
#[allow(unused_macros)]
mod macros;

pub mod commands;
pub mod messages;

pub mod irc;
pub use irc::{IrcMessage, MessageError};

#[doc(inline)]
pub use irc::{FromIrcMessage, IntoIrcMessage};

pub mod twitch;
pub use twitch::UserConfig;

mod decoder;
pub use decoder::{DecodeError, Decoder};

mod encoder;
pub use encoder::Encoder;

mod encodable;
pub use encodable::Encodable;

pub mod maybe_owned;
pub use maybe_owned::IntoOwned;
use maybe_owned::{MaybeOwned, MaybeOwnedIndex};

mod validator;
pub use validator::Validator;

#[cfg(feature = "serde")]
mod serde;
