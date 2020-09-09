use super::*;

/// A `tokio` connector that uses `tokio-native-tls` (a `native-tls` wrapper). This uses TLS.
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
    connector_ctor!(tls:
        /// [`tokio`](https://docs.rs/tokio/latest/tokio/) (using [`tokio-native-tls`](https://docs.rs/tokio-native-tls/latest/tokio_native_tls/))
    );
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
    fn assert_connector_trait_is_fulfilled() {
        use crate::connector::testing::*;
        use crate::connector::Connector as C;

        assert_connector::<ConnectorNativeTls>();
        assert_type_is_read_write::<<ConnectorNativeTls as C>::Output>();
        assert_obj_is_sane(ConnectorNativeTls::twitch());
    }
}
