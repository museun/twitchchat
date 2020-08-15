#![allow(
    clippy::missing_const_for_fn,
    clippy::redundant_pub_crate,
    clippy::use_self
)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
This crate provides a way to interface with [Twitch]'s chat.

Along with the messages as Rust types, it provides methods for sending messages.

---

[Twitch]: https://www.twitch.tv
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

#[macro_use]
mod maybe_owned;
pub use maybe_owned::{IntoOwned, MaybeOwned as Str, MaybeOwnedIndex as StrIndex};

mod decoder;
pub use decoder::{AsyncDecoder, Decoder, Error as DecodeError};

mod dispatcher;
pub use dispatcher::{AsyncDispatcher, DispatchError, SyncDispatcher};

mod encoder;
pub use encoder::{AsyncEncoder, Encodable, Encoder};

pub mod commands;
pub mod messages;

pub mod irc;
pub use irc::{InvalidMessage, IrcMessage, TagIndices, Tags};

#[doc(inline)]
pub use irc::FromIrcMessage;

mod validator;
pub use validator::Validator;

pub mod twitch;
#[doc(hidden)]
pub use twitch::*;

use twitch::color::Color;

mod rate_limit;
#[doc(inline)]
pub use rate_limit::RateClass;

#[cfg(feature = "serde")]
mod serde;

pub mod runner;
#[doc(inline)]
pub use runner::{AsyncRunner, Error as RunnerError};

pub mod writer;

pub mod channel;
#[doc(inline)]
pub use channel::{Receiver, Sender};

/// Asynchronous connectors for various runtimes.
pub mod connector;

// a public dep
pub use simple_event_map::{EventMap, EventStream};

// our internal stuff that should never be exposed
mod util;
