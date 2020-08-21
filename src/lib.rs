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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]
/*!

This crate provides a way to interface with [Twitch]'s chat (via IRC).

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

[Twitch]: https://www.twitch.tv
[runner]: runner/index.html
[encoder]: encoder/index.html
[decoder]: decoder/index.html
[twitch]: twitch/index.html
[messages]: messages/index.html
[commands]: commands/index.html
[irc]: irc/index.html
*/
// #[cfg(all(doctest, feature = "async", feature = "tokio_native_tls"))]
// doc_comment::doctest!("../README.md");

/// A boxed `Future` that is `Send + Sync`
pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + Sync>>;

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

// traits
#[doc(inline)]
pub use encoder::Encodable;
// #[doc(inline)]
pub use ext::PrivmsgExt;
#[doc(inline)]
pub use irc::{FromIrcMessage, IntoIrcMessage};
// #[doc(inline)]
pub use maybe_owned::IntoOwned;
#[doc(inline)]
pub use validator::Validator;

// errors
#[doc(inline)]
pub use decoder::DecodeError;
// #[doc(inline)]
pub use irc::MessageError;
// #[doc(inline)]
pub use runner::Error as RunnerError;

/// Prelude with common types
pub mod prelude {
    pub use super::decoder::{AsyncDecoder, Decoder};
    pub use super::encoder::{AsyncEncoder, Encodable, Encoder};
    pub use super::irc::{IrcMessage, TagIndices, Tags};
    pub use super::rate_limit::RateClass;
    pub use super::runner::{AsyncRunner, Identity, NotifyHandle, Status};
    pub use super::twitch;
    pub use super::{commands, messages};
}

#[macro_use]
#[allow(unused_macros)]
mod macros;

pub mod commands;
pub mod connector;
pub mod decoder;
pub mod encoder;
pub mod irc;
pub mod maybe_owned;
pub mod messages;
pub mod rate_limit;
pub mod runner;
pub mod twitch;

// TODO this could use more implementations and better documentation
pub mod writer;

// this is so we don't expose an external dep
pub mod channel;

#[doc(inline)]
pub use crate::prelude::{
    twitch::UserConfig, AsyncDecoder, AsyncEncoder, AsyncRunner, Decoder, Encoder, IrcMessage,
    Status,
};

mod validator;

use crate::channel::Sender;
use crate::maybe_owned::{MaybeOwned, MaybeOwnedIndex};
// use prelude::*;

// our internal stuff that should never be exposed
#[cfg(feature = "serde")]
mod serde;
mod util;

mod ext {
    use crate::{messages::Privmsg, Encodable};
    use std::io::Write;

    /// Extensions to the `Privmsg` message type
    pub trait PrivmsgExt {
        /// Reply to this message with `data` over `writer`
        fn reply<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
        where
            W: Write + ?Sized;

        /// Send a message back to the channel this Privmsg came from
        fn say<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
        where
            W: Write + ?Sized;
    }

    impl<'a> PrivmsgExt for Privmsg<'a> {
        fn reply<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
        where
            W: Write + ?Sized,
        {
            let cmd = crate::commands::reply(
                self.channel(),
                self.tags().get("id").ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "you must have `TAGS` enabled",
                    )
                })?,
                data,
            );
            cmd.encode(writer)?;
            writer.flush()
        }

        fn say<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
        where
            W: Write + ?Sized,
        {
            let cmd = crate::commands::privmsg(self.channel(), data);
            cmd.encode(writer)?;
            writer.flush()
        }
    }
}
