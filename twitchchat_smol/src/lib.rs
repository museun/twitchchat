use smol::net::{AsyncToSocketAddrs, TcpStream};

pub async fn connect_twitch() -> std::io::Result<TcpStream> {
    connect_custom(twitchchat::TWITCH_IRC_ADDRESS).await
}

pub async fn connect_custom<A>(addrs: A) -> std::io::Result<TcpStream>
where
    A: AsyncToSocketAddrs + Send + Sync,
    A::Iter: Send + Sync,
{
    TcpStream::connect(addrs).await
}

#[cfg(feature = "rustls")]
pub mod rustls {
    use async_tls::{client::TlsStream, TlsConnector};
    use smol::net::{AsyncToSocketAddrs, TcpStream};

    pub async fn connect_twitch() -> std::io::Result<TlsStream<TcpStream>> {
        connect_custom(
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            twitchchat::TWITCH_TLS_DOMAIN,
        )
        .await
    }

    pub async fn connect_custom<A, D>(addrs: A, domain: D) -> std::io::Result<TlsStream<TcpStream>>
    where
        A: AsyncToSocketAddrs + Send + Sync,
        A::Iter: Send + Sync,
        D: Into<String> + Send + Sync,
    {
        let stream = TcpStream::connect(addrs).await?;
        let domain = domain.into();
        TlsConnector::new().connect(&domain, stream).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_io::{AsyncRead, AsyncWrite};
    use std::future::Future;

    fn assert_it<F, Fut, R>(_func: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = std::io::Result<R>> + Send + 'static,
        R: AsyncRead + AsyncWrite,
        R: Send + Sync + Unpin + 'static,
    {
    }

    #[test]
    fn assert_non_tls_traits() {
        assert_it(connect_twitch);
        assert_it(|| async move {
            let addrs = "localhost".to_string();
            connect_custom(addrs).await
        });
    }

    #[cfg(feature = "rustls")]
    #[test]
    fn assert_tls_traits() {
        assert_it(rustls::connect_twitch);
        assert_it(|| async move {
            let addrs = "localhost".to_string();
            let domain = "localhost".to_string();
            rustls::connect_custom(addrs, domain).await
        });
    }
}
