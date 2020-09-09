use super::*;

/// A `tokio` connector that uses `tokio-rustls` (a `rustls` wrapper). This uses TLS.
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
    connector_ctor!(tls:
        /// [`tokio`](https://docs.rs/tokio/latest/tokio/) (using [`tokio-rustls`](https://docs.rs/tokio-rustls/latest/tokio_rustls/))
    );
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
    fn assert_connector_trait_is_fulfilled() {
        use crate::connector::testing::*;
        use crate::connector::Connector as C;

        assert_connector::<ConnectorRustTls>();
        assert_type_is_read_write::<<ConnectorRustTls as C>::Output>();
        assert_obj_is_sane(ConnectorRustTls::twitch());
    }
}
