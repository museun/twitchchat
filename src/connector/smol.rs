use crate::connector::try_connect;
use crate::BoxedFuture;

type TcpStream = smol::Async<std::net::TcpStream>;

/// A `smol` connector. This does not use TLS
#[derive(Debug, Clone, PartialEq)]
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
    type Output = TcpStream;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let addrs = self.addrs.clone();
        let fut = async move { try_connect(&*addrs, TcpStream::connect).await };
        Box::pin(fut)
    }
}

#[cfg(feature = "async-tls")]
pub use tls::*;

#[cfg(feature = "async-tls")]
mod tls {
    use super::*;

    /// A `smol` connector that uses `async-tls` (a `rustls` wrapper). This does use TLS.
    ///
    /// To use this type, ensure you set up the 'TLS Domain' in the
    /// configuration. The crate provides the 'TLS domain' for Twitch in the root of this crate.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConnectorTls {
        addrs: Vec<std::net::SocketAddr>,
        tls_domain: String,
    }

    impl ConnectorTls {
        /// Create a new `smol` TLS connector.
        pub fn twitch() -> Self {
            Self::custom(crate::TWITCH_IRC_ADDRESS_TLS, crate::TWITCH_TLS_DOMAIN)
                .expect("twitch DNS resolution")
        }

        /// Create a new `smol` TLS connector.
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
        type Output = async_tls::client::TlsStream<TcpStream>;

        fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
            let this = self.clone();
            let fut = async move {
                let stream = try_connect(&*this.addrs, TcpStream::connect).await?;
                async_tls::TlsConnector::new()
                    .connect(this.tls_domain, stream)
                    .await
            };
            Box::pin(fut)
        }
    }
}
