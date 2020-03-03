use crate::{register::register, simple_user_config, UserConfig, TWITCH_IRC_ADDRESS};

#[cfg(feature = "tls")]
use crate::TWITCH_IRC_ADDRESS_TLS;

type TokioStream = tokio::net::TcpStream;

#[cfg(feature = "tls")]
type TokioTlsStream = tokio_tls::TlsStream<tokio::net::TcpStream>;

#[cfg(feature = "tls")]
const TWITCH_DOMAIN: &str = "irc.chat.twitch.tv";

/**
Opens an ***async*** TCP connection using the provided `UserConfig`.

# Example
```rust,no_run
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let user_config = UserConfig::builder().anonymous().build().unwrap();
let stream = connect(&user_config).await.unwrap();
# });
```
*/
pub async fn connect(user_config: &UserConfig) -> std::io::Result<TokioStream> {
    let mut stream = tokio::net::TcpStream::connect(TWITCH_IRC_ADDRESS).await?;
    register(user_config, &mut stream).await?;
    Ok(stream)
}

/**
Opens an ***async*** TCP connection with _TLS_ using the provided `UserConfig`.

# Example
```rust,no_run
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let user_config = UserConfig::builder().anonymous().build().unwrap();
let stream = connect_tls(&user_config).await.unwrap();
# });
````
*/
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub async fn connect_tls(user_config: &UserConfig) -> std::io::Result<TokioTlsStream> {
    use std::io::{Error, ErrorKind};

    let stream = tokio::net::TcpStream::connect(TWITCH_IRC_ADDRESS_TLS).await?;
    let conn: tokio_tls::TlsConnector = native_tls::TlsConnector::new()
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .into();

    let mut stream = conn
        .connect(TWITCH_DOMAIN, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    register(user_config, &mut stream).await?;
    Ok(stream)
}

/**
Opens an ***async*** TCP connection using the provided `name`, `token`.

This enables all of the [Capabilities]

[Capabilities]: ./enum.Capability.html
# Example
```rust,no_run
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let (nick, pass) = ANONYMOUS_LOGIN;
let stream = connect_easy(nick, pass).await.unwrap();
# });
```
*/
pub async fn connect_easy(name: &str, token: &str) -> std::io::Result<TokioStream> {
    let config = simple_user_config(name, token).unwrap();
    connect(&config).await
}

/**
Opens an ***async*** TCP connection with _TLS_ using the provided `name`, `token`.

This enables all of the [Capabilities]

[Capabilities]: ./enum.Capability.html
# Example
```rust,no_run
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let (nick, pass) = ANONYMOUS_LOGIN;
let stream = connect_easy_tls(nick, pass).await.unwrap();
# });
```
*/
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub async fn connect_easy_tls(name: &str, token: &str) -> std::io::Result<TokioTlsStream> {
    let config = simple_user_config(name, token).unwrap();
    connect_tls(&config).await
}
