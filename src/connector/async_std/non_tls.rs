use super::*;

/// A `async_std` connector. This does not use TLS
#[derive(Debug, Clone, PartialEq)]
pub struct Connector {
    addrs: Vec<std::net::SocketAddr>,
}

impl Connector {
    connector_ctor!(non_tls:
        /// [`async-std`](https://docs.rs/async-std/latest/async_std/)
    );
}

impl crate::connector::Connector for Connector {
    type Output = async_std::net::TcpStream;

    fn connect(&mut self) -> BoxedFuture<std::io::Result<Self::Output>> {
        let addrs = self.addrs.clone();
        let fut = async move { async_std::net::TcpStream::connect(&*addrs).await };
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
