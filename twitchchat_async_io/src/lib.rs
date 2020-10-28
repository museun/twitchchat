use async_io::Async;
use std::net::{TcpStream, ToSocketAddrs};

pub async fn connect_twitch() -> std::io::Result<Async<TcpStream>> {
    connect_custom(twitchchat::TWITCH_IRC_ADDRESS).await
}

pub async fn connect_custom<A>(addrs: A) -> std::io::Result<Async<TcpStream>>
where
    A: ToSocketAddrs + Send + Sync,
    A::Iter: Send + Sync,
{
    for addr in addrs.to_socket_addrs()? {
        if let Ok(stream) = Async::<TcpStream>::connect(addr).await {
            return Ok(stream);
        }
    }

    Err(std::io::ErrorKind::AddrNotAvailable.into())
}

#[cfg(feature = "rustls")]
pub mod rustls {
    use async_io::Async;
    use async_tls::{client::TlsStream, TlsConnector};
    use std::net::{TcpStream, ToSocketAddrs};

    pub async fn connect_twitch() -> std::io::Result<TlsStream<Async<TcpStream>>> {
        connect_custom(
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            twitchchat::TWITCH_TLS_DOMAIN,
        )
        .await
    }

    pub async fn connect_custom<A, D>(
        addrs: A,
        domain: D,
    ) -> std::io::Result<TlsStream<Async<TcpStream>>>
    where
        A: ToSocketAddrs + Send + Sync,
        A::Iter: Send + Sync,
        D: Into<String> + Send + Sync,
    {
        let domain = domain.into();
        for addr in addrs.to_socket_addrs()? {
            if let Ok(stream) = Async::<TcpStream>::connect(addr).await {
                return TlsConnector::new().connect(&domain, stream).await;
            }
        }

        Err(std::io::ErrorKind::AddrNotAvailable.into())
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
        Fut: Future<Output = std::io::Result<R>> + Send + Sync + 'static,
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
