use super::*;

/// A `async_io` connector. This does not use TLS
#[derive(Debug, Clone, PartialEq)]
pub struct Connector {
    addrs: Vec<std::net::SocketAddr>,
}

impl Connector {
    connector_ctor!(non_tls:
        /// [`async_io`](https://docs.rs/async-io/latest/async_io/)
    );
}

impl crate::connector::Connector for Connector {
    type Output = TcpStream;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let addrs = self.addrs.clone();
        let fut = async move { try_connect(&*addrs, TcpStream::connect).await };
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

        assert_connector::<Connector>();
        assert_type_is_read_write::<<Connector as C>::Output>();
        assert_obj_is_sane(Connector::twitch());
    }
}
