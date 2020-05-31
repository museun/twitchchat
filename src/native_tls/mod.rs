/*! [`tokio-native-tls`][native_tls] connector for using a TLS connection with Twitch.

[native_tls]: https://docs.rs/tokio-native-tls/0.1.0/tokio_native_tls/
*/
use crate::UserConfig;
type Stream = tokio_native_tls::TlsStream<tokio::net::TcpStream>;

const TWITCH_DOMAIN: &str = "irc.chat.twitch.tv";

/// Connect to Twitch using TLS via **native_tls**. Using the provided [`UserConfig`][UserConfig].
///
/// This registers with the connection before returning it.
///
/// [UserConfig]: ../struct.UserConfig.html
///
/// # Example
/// ```rust,no_run
/// # use twitchchat::*;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
/// let user_config = UserConfig::builder().anonymous().build()?;
/// let mut stream = twitchchat::native_tls::connect(&user_config).await?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// # }).unwrap();
/// ```
pub async fn connect(config: &UserConfig) -> std::io::Result<Stream> {
    use std::io::{Error, ErrorKind};

    let conn: tokio_native_tls::TlsConnector = native_tls::TlsConnector::new()
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .into();

    let stream = tokio::net::TcpStream::connect(crate::TWITCH_IRC_ADDRESS_TLS).await?;
    let mut stream = conn
        .connect(TWITCH_DOMAIN, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    crate::register(config, &mut stream).await?;

    Ok(stream)
}

/// Connect to Twitch using TLS via **native_tls**. Using the provided `name`, `token`.
///
/// This registers with the connection before returning it.
///
/// # Example
/// ```rust,no_run
/// # use twitchchat::*;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
/// let (name, token) = ANONYMOUS_LOGIN;
/// let mut stream = twitchchat::native_tls::connect_easy(&name, &token).await?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// # }).unwrap();
/// ```
pub async fn connect_easy(name: &str, token: &str) -> std::io::Result<Stream> {
    use std::io::{Error, ErrorKind};

    let conn: tokio_native_tls::TlsConnector = native_tls::TlsConnector::new()
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .into();

    let stream = tokio::net::TcpStream::connect(crate::TWITCH_IRC_ADDRESS_TLS).await?;
    let mut stream = conn
        .connect(TWITCH_DOMAIN, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    let config = crate::simple_user_config(name, token) //
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    crate::register(&config, &mut stream).await?;

    Ok(stream)
}
