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
    // missing_docs, // TODO re-enable this
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
twitchchat = { version = "0.15", features = ["async", "writer"] }
```

### Available features:

| feature       | effect                                                                         |
| ------------- | ------------------------------------------------------------------------------ |
| `async`       | provides the [asynchronous] module (and generally all of the `async` functions |
| `sink_stream` | provides the [stream] module (for use with `Sink+Stream`                       |
| `writer`      | this enables the [`writer::MpscWriter`]                                        |

### Connectors:

In version `0.14` a `Connector` trait was provided with some common implementations. This has been moved out into separate crates.

* [twitchchat_async_io][twitchchat_async_io]
* [twitchchat_async_net][twitchchat_async_net]
* [twitchchat_async_std][twitchchat_async_std]
* [twitchchat_smol][twitchchat_smol]
* [twitchchat_tokio][twitchchat_tokio]
* [twitchchat_tokio02][twitchchat_tokio02]

---

### Useful modules for decoding/parsing/encoding:

For Twitch types:
* [twitch]
* [messages]
* [commands]
---
For the 'irc' types underneath it all:
* [irc]
---

[twitchchat_async_io]: https://docs.rs/twitchchat_async_io/latest/twitchchat_async_io
[twitchchat_async_net]: https://docs.rs/twitchchat_async_net/latest/twitchchat_async_net
[twitchchat_async_std]: https://docs.rs/twitchchat_async_std/latest/twitchchat_async_std
[twitchchat_smol]: https://docs.rs/twitchchat_smol/latest/twitchchat_smol
[twitchchat_tokio]: https://docs.rs/twitchchat_tokio/latest/twitchchat_tokio
[twitchchat_tokio02]: https://docs.rs/twitchchat_tokio02/latest/twitchchat_tokio02
*/

/// The Twitch IRC address for non-TLS connections
///
/// `irc.chat.twitch.tv:6667`
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
///
/// `irc.chat.twitch.tv:6697`
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
///
/// `irc-ws.chat.twitch.tv:80`
pub const TWITCH_WS_ADDRESS: &str = "irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
///
/// `irc-ws.chat.twitch.tv:443`
pub const TWITCH_WS_ADDRESS_TLS: &str = "irc-ws.chat.twitch.tv:443";

/// A TLS domain for Twitch's websocket
///
/// `irc-ws.chat.twitch.tv`
pub const TWITCH_WS_TLS_DOMAIN: &str = "irc-ws.chat.twitch.tv";

/// A TLS domain for Twitch
///
/// `irc.chat.twitch.tv`
pub const TWITCH_TLS_DOMAIN: &str = "irc.chat.twitch.tv";

/// An anonymous login.
pub const ANONYMOUS_LOGIN: (&str, &str) = (JUSTINFAN1234, JUSTINFAN1234);
pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

#[macro_use]
#[allow(unused_macros)]
mod macros;

#[cfg(feature = "serde")]
mod serde;

pub mod maybe_owned;

pub mod commands;
pub mod messages;

pub mod irc;
pub mod twitch;

pub mod test;

pub use encodable::Encodable;
pub use ext::PrivmsgExt;
#[doc(inline)]
pub use irc::{FromIrcMessage, IntoIrcMessage};
pub use maybe_owned::IntoOwned;
pub use validator::Validator;

use maybe_owned::{MaybeOwned, MaybeOwnedIndex};

mod encodable;
mod identity;
mod validator;

mod ext;
mod util;

mod decoder;
mod encoder;

mod handshake;
mod io;

mod timeout;
mod wait_for;

#[cfg(feature = "writer")]
#[cfg_attr(docsrs, doc(cfg(feature = "writer")))]
pub mod writer;

pub mod sync;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod asynchronous;

#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
pub mod stream;

mod make_sure_features_flags_are_correct {
    #[cfg(all(feature = "sink_stream", not(feature = "async")))]
    compile_error!("`async` must be enabled when `sink_stream` is enabled");
}
