use super::*;

/// A `async_std` connector that uses `async-tls` (a `rustls` wrapper). This uses TLS.
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
    connector_ctor!(tls:
        /// [`async-std`](https://docs.rs/async-std/latest/async_std/)
    );
}

impl crate::connector::Connector for ConnectorTls {
    type Output = async_dup::Mutex<async_tls::client::TlsStream<async_std::net::TcpStream>>;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let this = self.clone();
        let fut = async move {
            let stream = async_std::net::TcpStream::connect(&*this.addrs).await?;
            async_tls::TlsConnector::new()
                .connect(this.tls_domain, stream)
                .await
                .map(async_dup::Mutex::new)
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

        assert_connector::<ConnectorTls>();
        assert_type_is_read_write::<<ConnectorTls as C>::Output>();
        assert_obj_is_sane(ConnectorTls::twitch());
    }
}
