use crate::BoxedFuture;

/// A `tokio` connector. This does not use TLS
pub struct Connector {
    addrs: Vec<std::net::SocketAddr>,
}

impl Connector {
    /// Create a Connector that connects to Twitch
    pub fn twitch() -> Self {
        Self::custom(crate::TWITCH_IRC_ADDRESS).expect("twitch DNS resolution")
    }

    /// Create a connector with provided address(es)
    pub fn custom<A: std::net::ToSocketAddrs>(addrs: A) -> std::io::Result<Self> {
        addrs.to_socket_addrs().map(|addrs| Self {
            addrs: addrs.collect(),
        })
    }
}

impl crate::connector::Connector for Connector {
    type Output = tokio_util::compat::Compat<tokio::net::TcpStream>;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let addrs = self.addrs.clone();
        let fut = async move {
            use tokio_util::compat::Tokio02AsyncReadCompatExt as _;
            let stream = tokio::net::TcpStream::connect(&*addrs).await?;
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
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConnectorTls {
        addrs: Vec<std::net::SocketAddr>,
        tls_domain: String,
    }

    impl ConnectorTls {
        /// Create a new `tokio` TLS connector.
        pub fn twitch() -> Self {
            Self::custom(crate::TWITCH_IRC_ADDRESS, crate::TWITCH_TLS_DOMAIN)
                .expect("twitch DNS resolution")
        }

        /// Create a new `tokio` TLS connector.
        pub fn custom<A, D>(addrs: A, domain: D) -> std::io::Result<Self>
        where
            A: std::net::ToSocketAddrs,
            D: Into<String>,
        {
            let tls_domain = domain.into();
            addrs.to_socket_addrs().map(|addrs| Self {
                addrs: addrs.collect(),
                tls_domain,
            })
        }
    }

    impl crate::connector::Connector for ConnectorTls {
        type Output =
            tokio_util::compat::Compat<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>;

        fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
            let this = self.clone();
            let fut = async move {
                use tokio_util::compat::Tokio02AsyncReadCompatExt as _;
                let domain = tokio_rustls::webpki::DNSNameRef::try_from_ascii_str(&this.tls_domain)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

                let connector: tokio_rustls::TlsConnector = std::sync::Arc::new({
                    let mut c = tokio_rustls::rustls::ClientConfig::new();
                    c.root_store
                        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
                    c
                })
                .into();

                let stream = tokio::net::TcpStream::connect(&*this.addrs).await?;
                let stream = connector.connect(domain, stream).await?;

                Ok(stream.compat())
            };
            Box::pin(fut)
        }
    }
}
