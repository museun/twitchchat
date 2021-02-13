use super::*;

use std::io::{Error, ErrorKind};

/// A `tokio` connector that uses `tokio-openssl` (an `openssl` wrapper). This uses TLS.
///
/// To use this type, ensure you set up the 'TLS Domain' in the configuration.
///
/// The crate provides the 'TLS domain' for Twitch in the root of this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectorOpenSsl {
    addrs: Vec<std::net::SocketAddr>,
    tls_domain: String,
}

impl ConnectorOpenSsl {
    connector_ctor!(tls:
        /// [`tokio`](https://docs.rs/tokio/0.2/tokio/) (using [`tokio-openssl`](https://docs.rs/tokio_openssl/latest/tokio_openssl/))
    );
}

type CloneStream<T> = async_dup::Mutex<tokio_util::compat::Compat<T>>;
type Stream = tokio_openssl::SslStream<tokio::net::TcpStream>;

impl crate::connector::Connector for ConnectorOpenSsl {
    type Output = CloneStream<Stream>;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let this = self.clone();

        let fut = async move {
            use tokio_util::compat::TokioAsyncReadCompatExt as _;

            let config = ::openssl::ssl::SslConnector::builder(::openssl::ssl::SslMethod::tls())
                .and_then(|c| c.build().configure())
                .map_err(|err| Error::new(ErrorKind::Other, err))?;

            let stream = tokio::net::TcpStream::connect(&*this.addrs).await?;
            let ssl = config.into_ssl(&this.tls_domain).map_err(|err| Error::new(ErrorKind::Other, err))?;
            let mut stream = tokio_openssl::SslStream::new(ssl, stream)
                .map_err(|err| Error::new(ErrorKind::Other, err))?;
            std::pin::Pin::new(&mut stream)
                .connect()
                .await
                .map_err(|err| Error::new(ErrorKind::Other, err))?;

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

        assert_connector::<ConnectorOpenSsl>();
        assert_type_is_read_write::<<ConnectorOpenSsl as C>::Output>();
        assert_obj_is_sane(ConnectorOpenSsl::twitch().unwrap());
    }
}
