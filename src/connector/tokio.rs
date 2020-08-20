use crate::BoxedFuture;

/// A `tokio` connector. This does not use TLS
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
    // this Mutex is required because async_dup::Arc only impls the traits for `for<'a> &'a T`
    type Output = async_dup::Mutex<tokio_util::compat::Compat<tokio::net::TcpStream>>;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let addrs = self.addrs.clone();
        let fut = async move {
            use tokio_util::compat::Tokio02AsyncReadCompatExt as _;
            let stream = tokio::net::TcpStream::connect(&*addrs).await?;
            Ok(async_dup::Mutex::new(stream.compat()))
        };
        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_connector_is_futures_traits() {
        use crate::connector::Connector as ConnectorTrait;
        use futures_lite::{AsyncRead, AsyncWrite};

        fn assert_connector<T: ConnectorTrait>() {}
        fn assert_type_is_read_write<T: AsyncRead + AsyncWrite>() {}
        fn assert_obj_is_sane<T>(_obj: T)
        where
            T: ConnectorTrait,
            T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
            for<'a> &'a T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
        {
        }

        assert_connector::<Connector>();
        assert_type_is_read_write::<<Connector as ConnectorTrait>::Output>();
        assert_obj_is_sane(Connector::twitch());
    }
}

#[cfg(all(feature = "tokio-native-tls", feature = "native-tls"))]
pub use self::native_tls::*;

#[cfg(all(feature = "tokio-native-tls", feature = "native-tls"))]
mod native_tls {
    use super::*;

    /// A `tokio` connector that uses `tokio-native-tls` (a `native-tls` wrapper). This does use TLS.
    ///
    /// To use this type, ensure you set up the 'TLS Domain' in the configuration.
    ///
    /// The crate provides the 'TLS domain' for Twitch in the root of this crate.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConnectorNativeTls {
        addrs: Vec<std::net::SocketAddr>,
        tls_domain: String,
    }

    impl ConnectorNativeTls {
        /// Create a new `tokio` TLS connector.
        pub fn twitch() -> Self {
            Self::custom(crate::TWITCH_IRC_ADDRESS_TLS, crate::TWITCH_TLS_DOMAIN)
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

    type CloneStream<T> = async_dup::Mutex<tokio_util::compat::Compat<T>>;
    type Stream = tokio_native_tls::TlsStream<tokio::net::TcpStream>;

    impl crate::connector::Connector for ConnectorNativeTls {
        type Output = CloneStream<Stream>;

        fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
            let this = self.clone();

            let fut = async move {
                use tokio_util::compat::Tokio02AsyncReadCompatExt as _;

                let connector: tokio_native_tls::TlsConnector = ::native_tls::TlsConnector::new()
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?
                    .into();

                let stream = tokio::net::TcpStream::connect(&*this.addrs).await?;
                let stream = connector
                    .connect(&this.tls_domain, stream)
                    .await
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

                Ok(async_dup::Mutex::new(stream.compat()))
            };
            Box::pin(fut)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn assert_connector_is_futures_traits() {
            use crate::connector::Connector as ConnectorTrait;
            use futures_lite::{AsyncRead, AsyncWrite};

            fn assert_connector<T: ConnectorTrait>() {}
            fn assert_type_is_read_write<T: AsyncRead + AsyncWrite>() {}
            fn assert_obj_is_sane<T>(_obj: T)
            where
                T: ConnectorTrait,
                T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
                for<'a> &'a T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
            {
            }

            assert_connector::<ConnectorNativeTls>();
            assert_type_is_read_write::<<ConnectorNativeTls as ConnectorTrait>::Output>();
            assert_obj_is_sane(ConnectorNativeTls::twitch());
        }
    }
}

#[cfg(all(feature = "tokio-rustls", feature = "webpki-roots"))]
pub use rustls::*;

#[cfg(all(feature = "tokio-rustls", feature = "webpki-roots"))]
mod rustls {
    use super::*;

    /// A `tokio` connector that uses `tokio-rustls` (a `rustls` wrapper). This does use TLS.
    ///
    /// To use this type, ensure you set up the 'TLS Domain' in the configuration.
    ///
    /// The crate provides the 'TLS domain' for Twitch in the root of this crate.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConnectorRustTls {
        addrs: Vec<std::net::SocketAddr>,
        tls_domain: String,
    }

    impl ConnectorRustTls {
        /// Create a new `tokio` TLS connector.
        pub fn twitch() -> Self {
            Self::custom(crate::TWITCH_IRC_ADDRESS_TLS, crate::TWITCH_TLS_DOMAIN)
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

    impl crate::connector::Connector for ConnectorRustTls {
        type Output = async_dup::Mutex<
            tokio_util::compat::Compat<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>,
        >;

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
                Ok(async_dup::Mutex::new(stream.compat()))
            };
            Box::pin(fut)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn assert_connector_is_futures_traits() {
            use crate::connector::Connector as ConnectorTrait;
            use futures_lite::{AsyncRead, AsyncWrite};

            fn assert_connector<T: ConnectorTrait>() {}
            fn assert_type_is_read_write<T: AsyncRead + AsyncWrite>() {}
            fn assert_obj_is_sane<T>(_obj: T)
            where
                T: ConnectorTrait,
                T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
                for<'a> &'a T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
            {
            }

            assert_connector::<ConnectorRustTls>();
            assert_type_is_read_write::<<ConnectorRustTls as ConnectorTrait>::Output>();
            assert_obj_is_sane(ConnectorRustTls::twitch());
        }
    }
}
