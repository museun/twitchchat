//! Decoding types and functions.
//!
//! A decoder lets you decode messages from an [`std::io::Read`], [`futures::AsyncRead`][read], for a [`futures::Stream`] in either an iterative fashion, or one-by-one.
//!
//! When not using the [`std::iter::Iterator`] (or [`futures::Stream`][stream]), you'll get a borrowed message from the reader that is valid until the next read.
//!
//! With the `std::iter::Iterator` (or `futures::Stream`) interface, it'll return an owned messages.
//!
//! This crate provides both ***sync*** (`std::iter::Iterator` based) and ***async*** (`futures::Stream` based) decoding.
//! * sync: [Decoder]
//! * async: [AsyncDecoder]
//! * stream: [StreamDecoder]
//!
//! # Borrowed messages
//! ```
//! let input = "@key1=val;key2=true :user!user@user PRIVMSG #some_channel :\x01ACTION hello world\x01\r\n";
//! let mut reader = std::io::Cursor::new(input.as_bytes());
//!
//! // you can either &mut borrow the reader, or let the Decoder take ownership.
//! // ff it takes ownership you can retrieve the inner reader later.
//! let mut decoder = twitchchat::sync::Decoder::new(&mut reader);
//!
//! // returns whether the message was valid
//! // this'll block until it can read a 'full' message (e.g. one delimited by `\r\n`).
//! let msg = decoder.read_message().unwrap();
//!
//! // msg is borrowed until the next `read_message()`
//! // you can turn a borrowed message into an owned message by using the twitchchat::IntoOwned trait.
//! use twitchchat::IntoOwned as _;
//! let owned: twitchchat::irc::IrcMessage<'static> = msg.into_owned();
//! ```
//!
//! # Owned messages
//! ```
//! let input = "@key1=val;key2=true :user!user@user PRIVMSG #some_channel :\x01ACTION hello world\x01\r\n";
//! let mut reader = std::io::Cursor::new(input.as_bytes());
//!
//! // you can either &mut borrow the reader, or let the Decoder take ownership.
//! // ff it takes ownership you can retrieve the inner reader later.
//! for msg in twitchchat::sync::Decoder::new(&mut reader) {
//!     // this yields whether the message was valid or not
//!     // this'll block until it can read a 'full' message (e.g. one delimited by `\r\n`).
//!
//!     // notice its already owned here (denoted by the 'static lifetime)
//!     let msg: twitchchat::irc::IrcMessage<'static> = msg.unwrap();
//! }
//! ```

mod error;
pub use error::DecodeError;

#[allow(clippy::module_inception)]
mod decoder;
pub use decoder::Decoder;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
mod async_decoder;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use async_decoder::AsyncDecoder;

#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
mod stream_decoder;

#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
pub use stream_decoder::{ReadMessage, StreamDecoder};
