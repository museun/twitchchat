// #![warn(
//     missing_docs,
//     missing_debug_implementations,
//     missing_copy_implementations,
//     trivial_casts,
//     trivial_numeric_casts,
//     unsafe_code,
//     unstable_features,
//     unused_import_braces,
//     unused_qualifications
// )]
// #![cfg_attr(docsrs, feature(doc_cfg))]
// /*!
// This crate provides a way to interface with [Twitch]'s chat.

// Along with the messages as Rust types, it provides methods for sending messages.

// # Demonstration
// See `examples/demo.rs` for a demo of the api

// ---
// Here's a quick link to the [Event Mapping](./struct.Dispatcher.html#a-table-of-mappings)

// [Twitch]: https://www.twitch.tv
// */
// #[cfg(all(doctest, feature = "async", feature = "tokio_native_tls"))]
// doc_comment::doctest!("../README.md");

// #[macro_use]
// #[doc(hidden)]
// pub mod macros;

// // mod runner;
// // pub use runner::{
// //     control::Control,
// //     dispatcher::Dispatcher,
// //     runner::{Connector, RetryStrategy, Runner},
// //     status::Status,
// //     stream::EventStream,
// // };

// mod register;
// #[doc(inline)]
// pub use register::{register, register_easy};

// pub mod decode;
// #[doc(inline)]
// pub use decode::{decode, decode_one};

// pub mod encode;
// #[doc(inline)]
// pub use encode::{Encoder, SyncMpscWriter, SyncWriter};

// pub use encode::{AsyncMpscWriter, AsyncWriter as Writer};

pub mod twitch;
// #[doc(inline)]
pub use twitch::*;

// pub mod events;

// pub mod sync;

// mod parse;
// pub use parse::Parse;

// mod as_owned;
// #[doc(inline)]
// pub use as_owned::{AsOwned, Reborrow};

// mod error;
// #[doc(inline)]
// pub use error::Error;

// /// The Twitch IRC address for non-TLS connections
// pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

// /// The Twitch IRC address for TLS connections
// pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

// /// The Twitch WebSocket address for non-TLS connections
// pub const TWITCH_WS_ADDRESS: &str = "ws://irc-ws.chat.twitch.tv:80";

// /// The Twitch WebSocket address for TLS connections
// pub const TWITCH_WS_ADDRESS_TLS: &str = "wss://irc-ws.chat.twitch.tv:443";

// /**
// An anonymous login.

// You won't be able to send messages, but you can join channels and read messages

// # usage
// ```rust
// # use twitchchat::{ANONYMOUS_LOGIN, UserConfig};
// let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
// let _config = UserConfig::builder()
//     .name(nick)
//     .token(pass)
//     .build()
//     .unwrap();
// ```
// */
// pub const ANONYMOUS_LOGIN: (&str, &str) = (JUSTINFAN1234, JUSTINFAN1234);
// pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

// fn simple_user_config(name: &str, token: &str) -> Result<UserConfig, UserConfigError> {
//     UserConfig::builder()
//         .name(name)
//         .token(token)
//         .capabilities(&[
//             Capability::Membership,
//             Capability::Tags,
//             Capability::Commands,
//         ])
//         .build()
// }

// #[doc(inline)]
// pub mod rate_limit;
// #[doc(inline)]
// pub use rate_limit::RateLimit;

// // mod connect;
// // pub use connect::{connect_easy_no_tls, connect_no_tls};

// // TODO make these show up in the doc.rs build
// #[cfg(feature = "tokio_rustls")]
// pub mod rustls;

// #[cfg(feature = "tokio_native_tls")]
// pub mod native_tls;

// TODO WIP
pub mod ng;
