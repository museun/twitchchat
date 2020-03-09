use crate::{register::register, simple_user_config, UserConfig, TWITCH_IRC_ADDRESS};

#[cfg(any(feature = "tokio_native_tls", feature = "tokio_rustls"))]
use crate::TWITCH_IRC_ADDRESS_TLS;

#[cfg(any(feature = "tokio_native_tls", feature = "tokio_rustls"))]
const TWITCH_DOMAIN: &str = "irc.chat.twitch.tv";

type TokioStream = tokio::net::TcpStream;

#[cfg(feature = "tokio_native_tls")]
type TokioTlsStream = tokio_tls::TlsStream<tokio::net::TcpStream>;

#[cfg(feature = "tokio_rustls")]
type TokioTlsStream = tokio_rustls::client::TlsStream<tokio::net::TcpStream>;

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
```
*/
#[cfg(any(feature = "tokio_native_tls", feature = "tokio_rustls"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio_native_tls", feature = "tokio_rustls")))
)]
pub async fn connect_tls(user_config: &UserConfig) -> std::io::Result<TokioTlsStream> {
    let stream = tokio::net::TcpStream::connect(TWITCH_IRC_ADDRESS_TLS).await?;
    let mut stream = tls_connect(stream).await?;
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
#[cfg(any(feature = "tokio_native_tls", feature = "tokio_rustls"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio_native_tls", feature = "tokio_rustls")))
)]
pub async fn connect_easy_tls(name: &str, token: &str) -> std::io::Result<TokioTlsStream> {
    let config = simple_user_config(name, token).unwrap();
    connect_tls(&config).await
}

#[cfg(feature = "tokio_native_tls")]
async fn tls_connect(stream: TokioStream) -> std::io::Result<TokioTlsStream> {
    use std::io::{Error, ErrorKind};

    let conn: tokio_tls::TlsConnector = native_tls::TlsConnector::new()
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .into();

    let stream = conn
        .connect(TWITCH_DOMAIN, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(stream)
}

#[cfg(feature = "tokio_rustls")]
async fn tls_connect(stream: TokioStream) -> std::io::Result<TokioTlsStream> {
    // This isn't actually used by rustls, but is required 'for the future'.
    let domain_name = tokio_rustls::webpki::DNSNameRef::try_from_ascii_str(TWITCH_DOMAIN)
        .expect("valid twitch domain dns/ref");

    // Not sure which default roots to use, so lets trust the server
    let mut config = tokio_rustls::rustls::ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc(config));
    let stream = connector
        .connect(domain_name, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(stream)
}
