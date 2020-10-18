use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::compat::Compat;

pub async fn connect_twitch() -> std::io::Result<Compat<TcpStream>> {
    connect_custom(twitchchat::TWITCH_IRC_ADDRESS).await
}

pub async fn connect_custom(addrs: impl ToSocketAddrs) -> std::io::Result<Compat<TcpStream>> {
    use tokio_util::compat::Tokio02AsyncReadCompatExt;
    TcpStream::connect(addrs)
        .await
        .map(Tokio02AsyncReadCompatExt::compat)
}

#[cfg(feature = "native_tls")]
pub mod native_tls {
    use std::io::{Error, ErrorKind};
    use tokio::net::{TcpStream, ToSocketAddrs};
    use tokio_native_tls::{TlsConnector, TlsStream};
    use tokio_util::compat::Compat;

    pub async fn connect_twitch() -> std::io::Result<Compat<TlsStream<TcpStream>>> {
        connect_custom(
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            twitchchat::TWITCH_TLS_DOMAIN,
        )
        .await
    }

    pub async fn connect_custom(
        addrs: impl ToSocketAddrs,
        domain: impl Into<String>,
    ) -> std::io::Result<Compat<TlsStream<TcpStream>>> {
        use tokio_util::compat::Tokio02AsyncReadCompatExt;

        let connector: TlsConnector = ::native_tls::TlsConnector::new()
            .map_err(|err| Error::new(ErrorKind::Other, err))?
            .into();

        let stream = TcpStream::connect(addrs).await?;
        connector
            .connect(&domain.into(), stream)
            .await
            .map(Tokio02AsyncReadCompatExt::compat)
            .map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

#[cfg(feature = "rustls")]
pub mod rustls {
    use std::io::{Error, ErrorKind};
    use tokio::net::{TcpStream, ToSocketAddrs};
    use tokio_rustls::{client::TlsStream, rustls::ClientConfig, webpki::DNSNameRef, TlsConnector};
    use tokio_util::compat::Compat;

    pub async fn connect_twitch() -> std::io::Result<Compat<TlsStream<TcpStream>>> {
        connect_custom(
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            twitchchat::TWITCH_TLS_DOMAIN,
        )
        .await
    }

    pub async fn connect_custom(
        addrs: impl ToSocketAddrs,
        domain: impl Into<String>,
    ) -> std::io::Result<Compat<TlsStream<TcpStream>>> {
        use tokio_util::compat::Tokio02AsyncReadCompatExt;

        let domain = domain.into();
        let domain = DNSNameRef::try_from_ascii_str(&domain)
            .map_err(|err| Error::new(ErrorKind::Other, err))?;

        let connector: TlsConnector = std::sync::Arc::new({
            let mut c = ClientConfig::new();
            c.root_store
                .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
            c
        })
        .into();

        let stream = TcpStream::connect(addrs).await?;
        connector
            .connect(domain, stream)
            .await
            .map(Tokio02AsyncReadCompatExt::compat)
    }
}

#[cfg(feature = "openssl")]
pub mod openssl {
    use ::openssl10::ssl::{SslConnector, SslMethod};
    use std::io::{Error, ErrorKind};
    use tokio::net::{TcpStream, ToSocketAddrs};
    use tokio_openssl::SslStream;
    use tokio_util::compat::Compat;

    pub async fn connect_twitch() -> std::io::Result<Compat<SslStream<TcpStream>>> {
        connect_custom(
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            twitchchat::TWITCH_TLS_DOMAIN,
        )
        .await
    }

    pub async fn connect_custom(
        addrs: impl ToSocketAddrs,
        domain: impl Into<String>,
    ) -> std::io::Result<Compat<SslStream<TcpStream>>> {
        use tokio_util::compat::Tokio02AsyncReadCompatExt;

        let config = SslConnector::builder(SslMethod::tls())
            .and_then(|c| c.build().configure())
            .map_err(|err| Error::new(ErrorKind::Other, err))?;

        let stream = TcpStream::connect(addrs).await?;
        tokio_openssl::connect(config, &*domain.into(), stream)
            .await
            .map(Tokio02AsyncReadCompatExt::compat)
            .map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_io::{AsyncRead, AsyncWrite};
    use std::future::Future;

    fn assert_it<F, Fut, R>(_func: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = std::io::Result<R>> + Send + 'static,
        R: AsyncRead + AsyncWrite,
        R: Send + Sync + Unpin + 'static,
    {
    }

    #[test]
    fn assert_non_tls_traits() {
        assert_it(connect_twitch);
        assert_it(|| async move {
            let addrs = "localhost".to_string();
            connect_custom(addrs).await
        });
    }

    #[cfg(feature = "rustls")]
    #[test]
    fn assert_rustls_traits() {
        assert_it(rustls::connect_twitch);
        assert_it(|| async move {
            let addrs = "localhost".to_string();
            let domain = "localhost".to_string();
            rustls::connect_custom(addrs, domain).await
        });
    }

    #[cfg(feature = "native_tls")]
    #[test]
    fn assert_native_tls_traits() {
        assert_it(native_tls::connect_twitch);
        assert_it(|| async move {
            let addrs = "localhost".to_string();
            let domain = "localhost".to_string();
            native_tls::connect_custom(addrs, domain).await
        });
    }

    #[cfg(feature = "openssl")]
    #[test]
    fn assert_openssl_traits() {
        assert_it(openssl::connect_twitch);
        assert_it(|| async move {
            let addrs = "localhost".to_string();
            let domain = "localhost".to_string();
            openssl::connect_custom(addrs, domain).await
        });
    }
}
