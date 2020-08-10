use crate::connector::ConnectorConfig;
use crate::BoxedFuture;

/// A `async_std` connector. This does not use TLS
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
    type Output = async_std::net::TcpStream;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let config = self.config.clone();
        let fut = async move { async_std::net::TcpStream::connect(&*config.addrs).await };
        Box::pin(fut)
    }
}

#[cfg(feature = "async-tls")]
pub use tls::*;

#[cfg(feature = "async-tls")]
mod tls {
    use super::*;

    /// A `async_std` connector that uses `async-tls` (a `rustls` wrapper). This does use TLS.
    ///
    /// To use this type, ensure you set up the 'TLS Domain' in the configuration.
    ///
    /// The crate provides the 'TLS domain' for Twitch in the root of this crate.
    pub struct ConnectorTls {
        config: ConnectorConfig,
    }

    impl ConnectorTls {
        /// Create a new `async_std` TLS connector.
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
        type Output = async_tls::client::TlsStream<async_std::net::TcpStream>;

        fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
            let config = self.config.clone();
            let fut = async move {
                let stream = async_std::net::TcpStream::connect(&*config.addrs).await?;
                async_tls::TlsConnector::new()
                    .connect(config.tls_domain, stream)
                    .await
            };

            Box::pin(fut)
        }
    }
}
