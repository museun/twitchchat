#![cfg_attr(debug_assertions, allow(dead_code,))]
/*!
This crate provides a way to interace with [Twitch]'s chat.

Along with the messages as Rust types, it provides methods for sending messages.

# Simple example
```rust,ignore
#[tokio::main]
async fn main() {
    use futures::prelude::*;

    let (nick, pass) = (
        // twitch name
        std::env::var("TWITCH_NICK").unwrap(),
        // oauth token for twitch name
        std::env::var("TWITCH_PASS").unwrap(),
    );

    // putting this in the env so people don't join my channel when running this
    let channel = std::env::var("TWITCH_CHANNEL").unwrap();

    // connect via (tls or normal, 'Secure' determines that) tcp with this nick and password
    let (read, write) = twitchchat::connect_easy(&nick, &pass, twitchchat::Secure::Nope)
        .await
        .unwrap();

    // make a client. the client is clonable
    let client = twitchchat::Client::new();

    // get a future that resolves when the client is done reading, fails to read/write or is stopped
    let done = client.run(read, write);

    // get an event dispatcher
    let mut dispatcher = client.dispatcher().await;

    // subscribe to an event stream

    // for privmsg (what users send to channels)
    let mut privmsg = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // spawn a task to consume the stream
    tokio::task::spawn(async move {
        while let Some(msg) = privmsg.next().await {
            eprintln!("[{}] {}: {}", msg.channel, msg.user, msg.data);
        }
    });

    // for join (when a user joins a channel)
    let mut join = dispatcher.subscribe::<twitchchat::events::Join>();
    tokio::task::spawn(async move {
        while let Some(msg) = join.next().await {
            // we've joined a channel
            if msg.user == nick {
                eprintln!("you joined {}", msg.channel);
                break; // returning/dropping the stream un-subscribes it
            }
        }
    });

    // for privmsg again
    let mut bot = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // we can move the client to another task by cloning it
    let bot_client = client.clone();
    tokio::task::spawn(async move {
        let mut writer = bot_client.writer();
        while let Some(msg) = bot.next().await {
            match msg.data.split(" ").next() {
                Some("!quit") => {
                    // causes the client to shutdown
                    bot_client.stop().await.unwrap();
                }
                Some("!hello") => {
                    let response = format!("hello {}!", msg.user);
                    // send a message in response
                    let still_connected = writer.privmsg(&msg.channel, &response).await;
                    if !still_connected {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    // dispatcher has an RAII guard, so keep it scoped
    // dropping it here so everything can proceed while keeping example brief
    drop(dispatcher);

    // get a clonable writer from the client
    // join a channel, methods on writer return false if the client is disconnected
    if !client.writer().join(&channel).await {
        panic!("not connected!?")
    }

    // you can clear subscriptions with
    // client.dispatcher().await.clear_subscriptions::<event::Join>()
    // or all subscriptions
    // client.dispatcher().await.clear_subscriptions_all()

    // you can get the number of active subscriptions with
    // client.dispatcher().await.count_subscribers::<event::Join>()
    // or all subscriptions
    // client.dispatcher().await.count_subscribers_all()

    // await for the client to be done
    match done.await {
        Ok(twitchchat::client::Status::Eof) => {
            eprintln!("done!");
        }
        Ok(twitchchat::client::Status::Canceled) => {
            eprintln!("client was stopped by user");
        }
        Err(err) => {
            eprintln!("error: {}", err);
        }
    }

    // note you should wait for all of your tasks to join before exiting
    // but we detached them to make this shorter

    // another way would be to clear all subscriptions
    // clearing the subscriptions would close each event stream
    client.dispatcher().await.clear_subscriptions_all();
}
```

[Twitch]: https://www.twitch.tv
*/

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

#[macro_use]
mod macros;

cfg_async! {
    pub mod client;
    #[doc(inline)]
    pub use client::Client;
}

/// Decode messages from a `&str`
pub mod decode;
#[doc(inline)]
pub use decode::{decode, decode_many};

/// Encode messages to a writer
pub mod encode;
#[doc(inline)]
pub use encode::Encodable;

cfg_async! {
    #[doc(inline)]
    pub use encode::encode;
}

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

const TWITCH_DOMAIN: &str = "irc.chat.twitch.tv";

cfg_async! {
    use tokio::io::{AsyncRead, AsyncWrite};
}

/// Connection type
///
/// Defaults to `Nope`
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
    pub fn get_address(&self) -> &'static str {
        match self {
            Secure::UseTls => TWITCH_IRC_ADDRESS_TLS,
            Secure::Nope => TWITCH_IRC_ADDRESS,
        }
    }
}

cfg_async! {
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
    pub async fn register<W: ?Sized>(
        user_config: &UserConfig,
        writer: &mut W,
    ) -> std::io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        encode(user_config, writer).await
    }
}

cfg_async! {
    type ConnectRes = std::io::Result<(
        BoxAsyncRead, BoxAsyncWrite
    )>;

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
    pub async fn connect(
        user_config: &UserConfig,
        secure: impl Into<Option<Secure>>,
    ) -> ConnectRes {
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
            },
            Secure::Nope => {
                let mut stream = connect_no_tls(addr).await?;
                register(user_config, &mut stream).await?;
                let (read, write) = tokio::io::split(stream);
                Ok((Box::new(read), Box::new(write)))
            },
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

mod internal;
pub use internal::StringMarker;

/// Synchronous methods
pub mod sync;

/// A trait for converting crate types between `Owned` and `Borrowed` representations
///
/// # Example
/// ```rust
/// # use twitchchat::*;
/// # use twitchchat::messages::*;
/// let input = ":test!test@test JOIN #museun\r\n";
/// let message : Raw<&str> = decode::decode_many(&input).next().unwrap().unwrap();
/// let message_owned : Raw<String> = message.as_owned();
///
/// let join : Join<&str> = Join::parse(&message).unwrap();
/// let owned : Join<String> = join.as_owned();
/// let borrowed : Join<&str> = join.as_borrowed();
///
/// assert_eq!(borrowed, join);
/// ```
pub trait Conversion<'a> {
    /// The borrowed type
    type Borrowed: 'a;
    /// The owned type
    type Owned;

    /// Get a borrowed version
    fn as_borrowed(&'a self) -> Self::Borrowed;
    /// Get an owned version
    fn as_owned(&self) -> Self::Owned;
}

/// A trait for parsing messages
///
/// # Example
/// ```rust
/// # use twitchchat::*;
/// # use twitchchat::messages::*;
/// let input = ":test!test@test JOIN #museun\r\n";
/// let message : Raw<&str> = decode::decode_many(&input).next().unwrap().unwrap();
/// let join : Join<&str> = Join::parse(&message).unwrap();
/// assert_eq!(join, Join { channel: "#museun", user: "test" });
/// ```
pub trait Parse<T>
where
    Self: Sized,
    Self: crate::internal::private::parse_marker::Sealed<T>,
{
    /// Tries to parse the input as this message
    fn parse(input: T) -> Result<Self, crate::messages::InvalidMessage>;
}
