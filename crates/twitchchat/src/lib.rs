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

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "async")]
            #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
            $item
        )*
    };
}

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
pub const TWITCH_WS_ADDRESS: &str = "irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
pub const TWITCH_WS_ADDRESS_TLS: &str = "irc-ws.chat.twitch.tv:443";

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
cfg_async! {
    pub use decoder::AsyncDecoder;
    /// A boxed `AsyncRead` trait
    pub type BoxedRead = Box<dyn futures_lite::AsyncRead + Send + Sync + Unpin>;
    /// A boxed `AsyncDecoder`
    pub type BoxedAsyncDecoder = AsyncDecoder<BoxedRead>;
}

pub mod encoder;
pub use encoder::Encoder;
cfg_async! {
    pub use encoder::AsyncEncoder;
    /// A boxed `BoxedWrite` trait
    pub type BoxedWrite = Box<dyn futures_lite::AsyncWrite + Send + Sync + Unpin>;
    /// A boxed `AsyncEncoder`
    pub type BoxedAsyncEncoder = AsyncEncoder<BoxedWrite>;
}

cfg_async! {
/// Split an Async IO object and return Decoder/Encoder
pub fn make_pair<IO>(io: IO) -> (AsyncDecoder<futures_lite::io::ReadHalf< IO>>, AsyncEncoder<futures_lite::io::WriteHalf<IO>>)
where
    IO: futures_lite::AsyncRead + futures_lite::AsyncWrite,
    IO: Send + Sync + Unpin + 'static,
{
    let (read, write) = futures_lite::io::split(io);
    (AsyncDecoder::new(read), AsyncEncoder::new(write))
}
}

cfg_async! {
/// Split an Async IO object and return type-erased Decoder/Encoder
pub fn make_boxed_pair<IO>(io: IO) -> (BoxedAsyncDecoder, BoxedAsyncEncoder)
where
    IO: futures_lite::AsyncRead + futures_lite::AsyncWrite,
    IO: Send + Sync + Unpin + 'static,
{
    let (read, write) = futures_lite::io::split(io);
    let read: BoxedRead = Box::new(read);
    let write: BoxedWrite = Box::new(write);
    (AsyncDecoder::new(read), AsyncEncoder::new(write))
}
}

#[cfg(feature = "writer")]
#[allow(missing_docs)]
pub mod writer {
    use super::*;

    cfg_async! { use futures_lite::io::AsyncWrite; }

    use std::io::Write;

    enum Packet {
        Data(Box<[u8]>),
        Quit,
    }

    #[derive(Clone)]
    pub struct MpscWriter {
        tx: flume::Sender<Packet>,
        wait_for_it: flume::Receiver<()>,
    }

    impl std::fmt::Debug for MpscWriter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MpscWriter").finish()
        }
    }

    impl MpscWriter {
        pub fn from_encoder<W>(encoder: Encoder<W>) -> Self
        where
            W: Write + Send + Sync + 'static,
        {
            let mut encoder = encoder;
            let (tx, rx) = flume::unbounded();
            let (stop_tx, wait_for_it) = flume::bounded(1);
            Self::_start(rx, stop_tx, move |data| encoder.encode(data));
            Self { tx, wait_for_it }
        }

        cfg_async! {
        pub fn from_async_encoder<W>(encoder: AsyncEncoder<W>) -> Self
        where
            W: AsyncWrite + Send + Sync + Unpin + 'static,
        {
            let mut encoder = encoder;
            use futures_lite::future::block_on;
            let (tx, rx) = flume::unbounded();
            let (stop_tx, wait_for_it) = flume::bounded(1);
            Self::_start(rx, stop_tx, move |data| block_on(encoder.encode(data)));
            Self { tx, wait_for_it }
        }
        }

        fn _start<F>(receiver: flume::Receiver<Packet>, stop: flume::Sender<()>, mut encode: F)
        where
            F: FnMut(Box<[u8]>) -> std::io::Result<()>,
            F: Send + Sync + 'static,
        {
            // TODO don't do this for the async one
            let _ = std::thread::spawn(move || {
                let _stop = stop;
                for msg in receiver {
                    match msg {
                        Packet::Data(data) => encode(data)?,
                        Packet::Quit => break,
                    }
                }
                std::io::Result::Ok(())
            });
        }

        pub fn send(&self, enc: impl Encodable) -> std::io::Result<()> {
            let mut buf = Vec::new();
            enc.encode(&mut buf)?;

            if let Err(flume::TrySendError::Disconnected(..)) =
                self.tx.try_send(Packet::Data(buf.into_boxed_slice()))
            {
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            Ok(())
        }

        cfg_async! {
        pub async fn shutdown(self) {
            let _ = self.send(crate::commands::raw("QUIT\r\n"));
            let _ = self.tx.try_send(Packet::Quit);
            let _ = self.wait_for_it.into_recv_async().await;
        }
        }

        pub fn shutdown_sync(self) {
            let _ = self.send(crate::commands::raw("QUIT\r\n"));
            let _ = self.tx.try_send(Packet::Quit);
            let _ = self.wait_for_it.recv();
        }
    }
}

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
