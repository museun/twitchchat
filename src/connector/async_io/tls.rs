use super::*;
use std::io::Result;

/// A `async_io` connector that uses `async-tls` (a `rustls` wrapper). This uses TLS.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectorTls {
    addrs: Vec<std::net::SocketAddr>,
    tls_domain: String,
}

impl ConnectorTls {
    connector_ctor!(tls:
        /// [`async_io`](https://docs.rs/async-io/latest/async_io/)
    );
}

impl crate::connector::Connector for ConnectorTls {
    type Output = async_dup::Mutex<async_tls::client::TlsStream<TcpStream>>;

    fn connect(&mut self) -> BoxedFuture<Result<Self::Output>> {
        let this = self.clone();
        let fut = async move {
            let stream = try_connect(&*this.addrs, TcpStream::connect).await?;
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
