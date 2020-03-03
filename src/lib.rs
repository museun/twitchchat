#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
This crate provides a way to interface with [Twitch]'s chat.

Along with the messages as Rust types, it provides methods for sending messages.

# Demonstration
See `examples/demo.rs` for a demo of the api

[Twitch]: https://www.twitch.tv
*/

#[macro_use]
#[doc(hidden)]
pub mod macros;

cfg_async! {
    use tokio::io::{AsyncRead, AsyncWrite};
}

cfg_async! {
    pub mod client;
    pub use client::Client;
}

/// Decode messages from a `&str`
pub mod decode;
#[doc(inline)]
pub use decode::{decode, decode_one};

mod encode;
#[doc(inline)]
pub use encode::Encoder;

/// Common Twitch types
pub mod twitch;

#[doc(inline)]
pub use twitch::*;

cfg_async! {
    pub mod events;
}

pub mod messages;

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
pub const TWITCH_WS_ADDRESS: &str = "ws://irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
pub const TWITCH_WS_ADDRESS_TLS: &str = "wss://irc-ws.chat.twitch.tv:443";

cfg_async! {
    /// Connection type
    ///
    /// Defaults to `Nope` (no TLS)
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Secure {
        /// Use TLS
        UseTls,
        /// Don't use TLS
        Nope,
    }

    impl Default for Secure {
        fn default() -> Self {
            Self::Nope
        }
    }

    impl Secure {
        /// Gets the requested (IRC) address
        pub fn get_address(self) -> &'static str {
            match self {
                Self::UseTls => TWITCH_IRC_ADDRESS_TLS,
                Self::Nope => TWITCH_IRC_ADDRESS,
            }
        }
    }
}

cfg_async! {
    mod register {
        use super::*;
        /**
        Write the provided `UserConfig` to the ***async*** writer

        # Example
        ```rust
        # use twitchchat::*;
        # tokio::runtime::Runtime::new().unwrap().block_on(async move {
        let config = UserConfig::builder().anonymous().build().unwrap();
        let mut writer = vec![];
        register(&config, &mut writer).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&writer).unwrap(),
            "PASS justinfan1234\r\nNICK justinfan1234\r\n"
        );
        # });
        ```
        */
        pub async fn register<W: ?Sized>(user_config: &UserConfig, writer: &mut W) -> std::io::Result<()>
        where
            W: AsyncWrite + Unpin,
        {
            use tokio::io::AsyncWriteExt as _;

            let UserConfig {
                name,
                token,
                capabilities,
            } = user_config;

            for cap in capabilities {
                let cap = cap.encode_as_str();
                log::trace!("sending CAP: {}", cap);
                writer.write_all(cap.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }

            log::trace!("sending PASS: {} (len={})", "*".repeat(token.len()), token.len());
            log::trace!("sending NICK: {}", name);
            writer
                .write_all(format!("PASS {}\r\nNICK {}\r\n", token, name).as_bytes())
                .await?;

            log::trace!("flushing initial handshake");
            writer.flush().await.map_err(Into::into)
        }
    }
    #[doc(inline)]
    pub use register::register;
}

cfg_async! {
    const TWITCH_DOMAIN: &str = "irc.chat.twitch.tv";

    type ConnectRes = std::io::Result<(BoxAsyncRead, BoxAsyncWrite)>;

    async fn connect_no_tls(addr: &str) -> std::io::Result<impl AsyncWrite + AsyncRead + Unpin> {
        tokio::net::TcpStream::connect(addr).await
    }

    async fn connect_tls(addr: &str) -> std::io::Result<impl AsyncWrite + AsyncRead + Unpin> {
        use std::io::{Error, ErrorKind};
        let stream = tokio::net::TcpStream::connect(addr).await?;
        let conn: tokio_tls::TlsConnector = native_tls::TlsConnector::new()
            .map_err(|err| Error::new(ErrorKind::Other, err))?
            .into();
        conn.connect(TWITCH_DOMAIN, stream)
            .await
            .map_err(|err| Error::new(ErrorKind::Other, err))
    }

    /// Boxed AsyncRead
    pub type BoxAsyncRead = Box<dyn AsyncRead + Send + Sync + Unpin>;
    /// Boxed AsyncWrite
    pub type BoxAsyncWrite = Box<dyn AsyncWrite + Send + Sync + Unpin>;

    /**
    Opens an ***async*** TCP connection using the provided `UserConfig` and `Secure` setting

    If `secure` is `None`, it'll use the normal TCP socket

    # Panics
    This panics if you try to use a secure connection but have the `tls` feature disabled

    # Example
    ## With TLS
    ```rust
    # use twitchchat::*;
    # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    let user_config = UserConfig::builder().anonymous().build().unwrap();
    let secure = Secure::Nope; // or None
    let (read, write) = connect(&user_config, secure).await.unwrap();
    # });
    ```

    ## Without TLS
    ```rust
    # use twitchchat::*;
    # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    let user_config = UserConfig::builder().anonymous().build().unwrap();
    let secure = Secure::Nope; // or None
    let (read, write) = connect(&user_config, secure).await.unwrap();
    # });
    ```
    */
    pub async fn connect(user_config: &UserConfig, secure: impl Into<Option<Secure>>) -> ConnectRes {
        let secure = secure.into().unwrap_or_default();
        let addr = secure.get_address();

        match secure {
            Secure::UseTls => {
                if cfg!(feature = "tls") {
                    let mut stream = connect_tls(addr).await?;
                    register(user_config, &mut stream).await?;
                    let (read, write) = tokio::io::split(stream);
                    Ok((Box::new(read), Box::new(write)))
                } else {
                    panic!("enable the \"tls\" feature to use this")
                }
            }
            Secure::Nope => {
                let mut stream = connect_no_tls(addr).await?;
                register(user_config, &mut stream).await?;
                let (read, write) = tokio::io::split(stream);
                Ok((Box::new(read), Box::new(write)))
            }
        }
    }

    /**
    Opens an ***async*** TCP connection using the provided `name`, `token` and `Secure` setting

    If `secure` is `None`, it'll use the normal TCP socket

    This enables all of the [Capabilities]

    [Capabilities]: ./enum.Capability.html

    # Panics
    This panics if you try to use a secure connection but have the `tls` feature disabled

    # Example

    ## With TLS
    ```rust
    # use twitchchat::*;
    # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    let (nick, pass) = ANONYMOUS_LOGIN;
    let secure = Secure::UseTls;
    let (read, write) = connect_easy(nick, pass, secure).await.unwrap();
    # });
    ```

    ## Without TLS
    ```rust
    # use twitchchat::*;
    # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    let (nick, pass) = ANONYMOUS_LOGIN;
    let secure = Secure::Nope; // or None
    let (read, write) = connect_easy(nick, pass, secure).await.unwrap();
    # });
    ```
    */
    pub async fn connect_easy(
        name: &str,
        token: &str,
        secure: impl Into<Option<Secure>>,
    ) -> ConnectRes {
        let config = simple_user_config(name, token).unwrap();
        connect(&config, secure).await
    }
}

fn simple_user_config(name: &str, token: &str) -> Result<UserConfig, UserConfigError> {
    UserConfig::builder()
        .name(name)
        .token(token)
        .capabilities(&[
            Capability::Membership,
            Capability::Tags,
            Capability::Commands,
        ])
        .build()
}

/**
An anonymous login.

You won't be able to send messages, but you can join channels and read messages

# usage
```rust
# use twitchchat::{ANONYMOUS_LOGIN,UserConfig};
let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
let _config = UserConfig::builder()
    .name(nick)
    .token(pass)
    .build()
    .unwrap();
```
*/
pub const ANONYMOUS_LOGIN: (&str, &str) = (JUSTINFAN1234, JUSTINFAN1234);

pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

/// Synchronous methods
pub mod sync;

cfg_async! {
    #[doc(inline)]
    pub mod rate_limit;
}

/// A trait for parsing messages
///
/// # Example
/// ```rust
/// # use twitchchat::*;
/// # use twitchchat::messages::*;
/// # use std::borrow::Cow;
///
/// let input = ":test!test@test JOIN #museun\r\n";
/// let message: Raw<'_> = decode::decode(&input).next().unwrap().unwrap();
/// let join: Join<'_> = Join::parse(&message).unwrap();
/// assert_eq!(join, Join { channel: Cow::Borrowed("#museun"), name: Cow::Borrowed("test") });
/// ```
pub trait Parse<T>: Sized + private::ParseSealed<T> {
    /// Tries to parse the input as this message
    fn parse(input: T) -> Result<Self, crate::messages::InvalidMessage>;
}

mod as_owned;
#[doc(inline)]
pub use as_owned::AsOwned;

mod private {
    pub trait ParseSealed<E> {}
    impl<T: crate::Parse<E>, E: Sized> ParseSealed<E> for T {}
}
