//! # A simple "connection-less" twitch chat crate
//! This crate simply read lines from an [std::io::Read]() and produces data
//! types for the corresponding messages, and takes an [std::io::Write]() which
//! can produce messages that twitch understands.
//!
//! # Organization of project
//! This crate is split into two top-level modules, `irc` and `twitch`.
//!
//! The [`irc`](./irc/index.html) module contains a **very** simplistic
//! representation of the IRC protocol, but can be used to write simplistic
//! clients for interacting with the twitch module
//!
//! The [`twitch`](./twitch/index.html) module contains many data types that
//! encapsulate the various functionality of the irc-portion of Twitch's chat
//! system
//!
//! # 'Theory' of operation
//! First, by creating a [`Client`](./twitch/struct.Client.html) from a
//! Read/Write pair (such as a cloned TcpStream) then calling
//! [`Client::register`](./twitch/struct.Client.html#method.register) with a
//! filled-out [`UserConfig`](./struct.UserConfig.html) will connect you to
//! Twitch. Once connected, you can
//! [`Client::wait_for_ready`](./twitch/struct.Client.html#method.
//! wait_for_ready) and the client will read (blocking) until Twitch sends a
//! [`GlobalUserState`](twitch/commands/struct.GlobalUserState.html) message,
//! which it'll fill out a [`LocalUser`](./twitch/struct.LocalUser.html) with
//! various information.
//!
//! Once connected, you can
//! - use [`Client::join`](./twitch/struct.Client.html#method.join) to join a
//!   channel.
//! - use [`Client::on`](./twitch/struct.Client.html#method.on) to set up a
//!   message filter.
//! - use [`Client::read_message`](./twitch/struct.Client.html#method.
//!   read_message) to read a message (and pump the filters).
//! - or do various [*other things*](./twitch/struct.Client.html#method.host)
//!
//! # Message filters, and why blocking with this is a bad idea
//! The client provides a very simplistic callback registration system
//!
//! To use it, you simply just `register` a closure with the client via its
//! [`Client::on`](./twitch/struct.Client.html#method.on) method. It uses the
//! type of the closures argument, one of
//! [*these*](./twitch/commands/index.html#structs) to create a filter When
//! [`Client::read_message`](./twitch/struct.Client.html#method.read_message) is
//! called, it'll check these filters and send a clone of the requested message
//! to the callback. Because it does this on same thread as the
//! [`Client::read_message`](./twitch/struct.Client.html#method.read_message)
//! call, you can lock up the system by simplying diverging.
//!
//! The client is thread safe, and clonable so one could call
//! [`Client::read_message`](./twitch/struct.Client.html#method.read_message)
//! with ones own sychronization scheme to allow for a simplistic thread pool,
//! but its best just to send the message off to a channel elsehwere
//!
//! # A simple example
//! ```no_run
//! use std::net::TcpStream;
//! use twitchchat::twitch::{commands::PrivMsg, Capability, Client};
//! use twitchchat::UserConfig;
//! # fn main() {
//! // create a simple TcpStream
//! let read = TcpStream::connect("irc.chat.twitch.tv:6667").expect("to connect");
//! let write = read
//!     .try_clone()
//!     .expect("must be able to clone the tcpstream");
//!
//! // your password and your nickname
//! // the twitch oauth token must be prefixed with `oauth:your_token_here`
//! let (nick, pass) = (std::env::var("MY_TWITCH_OAUTH_TOKEN").unwrap(), "my_name");
//! let config = UserConfig {
//!     token: pass.to_string(),
//!     nick: nick.to_string(),
//!     caps: vec![
//!         Capability::Membership, // enable seeing whom is on the channel
//!         Capability::Commands,   // enable sending twitch-specific commands (and receiving them)
//!         Capability::Tags,       // enable metadata (`tags`) on many of the messages
//!     ],
//! };
//!
//! // client takes a std::io::Read and an std::io::Write
//! let mut client = Client::new(read, write);
//!
//! // register with the user configuration
//! client.register(config).unwrap();
//!
//! // wait for everything to be ready (blocks)
//! let user = client.wait_for_ready().unwrap();
//! println!(
//!     "connected with {} (id: {}). our color is: {}",
//!     user.display_name.unwrap(),
//!     user.user_id,
//!     user.color.unwrap_or_default()
//! );
//!
//! // when we receive a commands::PrivMsg print out who sent it, and the message
//! client.on(|msg: PrivMsg| println!("{}: {}", msg.display_name().unwrap(), msg.message()));
//!
//! // blocks the thread, but any callbacks set in the .on handlers will get their messages
//! client.run();
//! # }
//! ```
//!
//! # TestStream
//! [`TestStream`](./struct.TestStream.html) is a simple TcpStream-like thing
//! that lets you inject/read its internal buffers, allowing you to easily write
//! unit tests for the [`Client`](./twitch/struct.Client.html)
//!
//! # UserConfigBuilder
//! [`UserConfigBuilder`](./struct.UserConfigBuilder.html) gives you less of a
//! chance to mess up the order of strings in the [`UserConfig`](./struct.UserConfig.html).
//!
//! ```rust
//! # fn main {
//! let config = UserConfig::builder()
//!                 .token("my_secrets")
//!                 .nick("my_name")
//!                 .build()
//!                 .unwrap();
//! // config.nick = "my_nick"
//! // config.token = "my_secrets"
//! // config.caps = vec![Membership,Commands,Tags]
//! # }
//! ```

/// IRC-related stuff
pub mod irc;

/// Types associated with twitch
pub mod twitch;

mod userconfig;
pub use self::userconfig::{UserConfig, UserConfigBuilder};

#[cfg(feature = "teststream")]
mod teststream;

#[cfg(feature = "teststream")]
pub use teststream::TestStream;

#[allow(dead_code)]
pub(crate) const VERSION_STR: &str =
    concat!(env!("CARGO_PKG_NAME"), ":", env!("CARGO_PKG_VERSION"));

/// The twitch irc address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";
/// The twitch irc address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";
