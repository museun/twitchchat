/*! [`tokio-rustls`][rustls] connector for using a TLS connection with Twitch.

[rustls]: https://docs.rs/tokio-rustls/0.13.1/tokio_rustls/index.html
*/
use crate::UserConfig;
type Stream = tokio_rustls::client::TlsStream<tokio::net::TcpStream>;

const TWITCH_DOMAIN: &str = "irc.chat.twitch.tv";

/// Connect to Twitch using TLS via **rustls**. Using the provided [`UserConfig`][UserConfig].
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
/// let mut stream = twitchchat::rustls::connect(&user_config).await?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// # }).unwrap();
/// ```
pub async fn connect(config: &UserConfig) -> std::io::Result<Stream> {
    use std::io::{Error, ErrorKind};

    let domain_name = tokio_rustls::webpki::DNSNameRef::try_from_ascii_str(TWITCH_DOMAIN)
        .expect("valid twitch domain dns/ref");

    let mut tls_config = tokio_rustls::rustls::ClientConfig::new();
    tls_config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let stream = tokio::net::TcpStream::connect(crate::TWITCH_IRC_ADDRESS_TLS).await?;
    let mut stream = tokio_rustls::TlsConnector::from(std::sync::Arc::new(tls_config))
        .connect(domain_name, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    crate::register(config, &mut stream).await?;

    Ok(stream)
}

/// Connect to Twitch using TLS via **rustls**. Using the provided `name`, `token`.
///
/// This registers with the connection before returning it.
///
/// # Example
/// ```rust,no_run
/// # use twitchchat::*;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
/// let (name, token) = ANONYMOUS_LOGIN;
/// let mut stream = twitchchat::rustls::connect_easy(&name, &token).await?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// # }).unwrap();
/// ```
pub async fn connect_easy(name: &str, token: &str) -> std::io::Result<Stream> {
    use std::io::{Error, ErrorKind};

    let domain_name = tokio_rustls::webpki::DNSNameRef::try_from_ascii_str(TWITCH_DOMAIN)
        .expect("valid twitch domain dns/ref");

    let mut tls_config = tokio_rustls::rustls::ClientConfig::new();
    tls_config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let stream = tokio::net::TcpStream::connect(crate::TWITCH_IRC_ADDRESS_TLS).await?;
    let mut stream = tokio_rustls::TlsConnector::from(std::sync::Arc::new(tls_config))
        .connect(domain_name, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    let config = crate::simple_user_config(name, token) //
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    crate::register(&config, &mut stream).await?;

    Ok(stream)
}
