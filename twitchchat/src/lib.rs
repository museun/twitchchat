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
#![cfg_attr(docsrs, feature(broken_intra_doc_links))]
/*!

This crate provides a way to interface with [Twitch](https://dev.twitch.tv/docs/irc)'s chat (via IRC).

Along with the messages as Rust types, it provides methods for sending messages.
---

By default, this crate depends on zero external crates -- but it makes it rather limited in scope.

This allows parsing, and decoding/encoding to standard trait types (`std::io::{Read, Write}`).

```toml
twitchchat = { version = "0.15", features = ["async"] }
```
---

For twitch types:
* [twitch]
* [messages]
* [commands]
---
For the 'irc' types underneath it all:
* [irc]
---
For event loop helpers:
* [runner]
---
For just decoding messages:
* [decoder]
---
For just encoding messages:
* [encoder]
*/

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
pub const TWITCH_WS_ADDRESS: &str = "irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
pub const TWITCH_WS_ADDRESS_TLS: &str = "irc-ws.chat.twitch.tv:443";

/// A TLS domain for Twitch's websocket
pub const TWITCH_WS_TLS_DOMAIN: &str = "irc-ws.chat.twitch.tv";

/// A TLS domain for Twitch
pub const TWITCH_TLS_DOMAIN: &str = "irc.chat.twitch.tv";

/// An anonymous login.
pub const ANONYMOUS_LOGIN: (&str, &str) = (JUSTINFAN1234, JUSTINFAN1234);
pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

#[macro_use]
#[allow(unused_macros)]
mod macros;

pub mod decoder;
pub use decoder::{DecodeError, Decoder};
cfg_async! { pub use decoder::AsyncDecoder; }

pub mod encoder;
pub use encoder::Encoder;
cfg_async! { pub use encoder::AsyncEncoder; }

pub mod split;

cfg_writer! { pub mod writer; }

pub mod runner;
pub use runner::Error;

pub mod commands;
pub mod messages;

pub mod irc;
pub use irc::{IrcMessage, MessageError};

/// Helpful testing utilities
pub mod test;

#[doc(inline)]
pub use irc::{FromIrcMessage, IntoIrcMessage};

pub mod twitch;
pub use twitch::UserConfig;

mod encodable;
pub use encodable::Encodable;

pub mod maybe_owned;
pub use maybe_owned::IntoOwned;
use maybe_owned::{MaybeOwned, MaybeOwnedIndex};

mod validator;
pub use validator::Validator;

#[cfg(feature = "serde")]
mod serde;
mod util;

mod ext;
pub use ext::PrivmsgExt;
