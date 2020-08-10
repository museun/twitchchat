use crate::connector::ConnectorConfig;
use crate::BoxedFuture;

/// A `tokio` connector. This does not use TLS
pub struct Connector {
    config: ConnectorConfig,
}

impl Connector {
    /// Create a new connector with the provided configuration
    pub const fn new(config: ConnectorConfig) -> Self {
        Self { config }
    }
}

impl crate::connector::Connector for Connector {
    type Output = tokio_util::compat::Compat<tokio::net::TcpStream>;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let config = self.config.clone();
        let fut = async move {
            use tokio_util::compat::Tokio02AsyncReadCompatExt as _;
            let stream = tokio::net::TcpStream::connect(&*config.addrs).await?;
            Ok(stream.compat())
        };
        Box::pin(fut)
    }
}

#[cfg(feature = "tokio-rustls")]
pub use tls::*;

#[cfg(feature = "tokio-rustls")]
mod tls {
    use super::*;

    /// A `tokio` connector that uses `tokio-rustls` (a `rustls` wrapper). This does use TLS.
    ///
    /// To use this type, ensure you set up the 'TLS Domain' in the configuration.
    ///
    /// The crate provides the 'TLS domain' for Twitch in the root of this crate.
    pub struct ConnectorTls {
        config: ConnectorConfig,
    }

    impl ConnectorTls {
        /// Create a new `tokio` TLS connector.
        ///
        /// If the `TLS Domain` in the configuration is empty, this will return an error.
        ///
        /// If you're unsure of which TLS domain to use, use the one in the root of this crate.
        pub fn new(config: ConnectorConfig) -> std::io::Result<Self> {
            if config.tls_domain.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "you must provide a TLS domain",
                ));
            }
            Ok(Self { config })
        }
    }

    impl crate::connector::Connector for ConnectorTls {
        type Output =
            tokio_util::compat::Compat<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>;

        fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
            let config = self.config.clone();
            let fut = async move {
                use tokio_util::compat::Tokio02AsyncReadCompatExt as _;
                let domain =
                    tokio_rustls::webpki::DNSNameRef::try_from_ascii_str(&config.tls_domain)
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

                let connector: tokio_rustls::TlsConnector = std::sync::Arc::new({
                    let mut c = tokio_rustls::rustls::ClientConfig::new();
                    c.root_store
                        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
                    c
                })
                .into();

                let stream = tokio::net::TcpStream::connect(&*config.addrs).await?;
                let stream = connector.connect(domain, stream).await?;

                Ok(stream.compat())
            };
            Box::pin(fut)
        }
    }
}
