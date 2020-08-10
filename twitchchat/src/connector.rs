use futures_lite::{AsyncRead, AsyncWrite};
use std::{future::Future, io::Result as IoResult, net::SocketAddr};

#[cfg(feature = "async-io")]
/// Connector for using an `async_io` wrapper over `std::net::TcpStream`
pub mod async_io;

#[cfg(feature = "async-std")]
/// Connector for using an `async_std::net::TcpStream`
pub mod async_std;

#[cfg(feature = "smol")]
/// Connector for using an `smol` wrapper over `std::net::TcpStream`
pub mod smol;

#[cfg(all(feature = "tokio", feature = "tokio-util"))]
/// Connector for using an `tokio::net::TcpStream`
pub mod tokio;

/// The connector traits. This is used to abstract out runtimes.
///
/// You can implement this on your own type to provide a custom connection behavior.
pub trait Connector: Send + Sync + Clone {
    /// Output IO type returned by calling `connect`
    ///
    /// This type must implement `futures::io::AsyncRead` and `futures::io::AsyncWrite`
    type Output: AsyncRead + AsyncWrite + Send + Sync + 'static;
    /// The `connect` method. This should return a boxed future of a `std::io::Result` of the `Output` type.
    ///
    /// e.g. `Box::pin(async move { std::net::TcpStream::connect("someaddr") })
    fn connect(&mut self) -> crate::BoxedFuture<IoResult<Self::Output>>;
}

// This is used because smol/async_io uses an indv. SocketAddr for their connect
// instead of the normal ToSocketAddrs trait
//
// thus this will be dead if those features aren't enabled.
#[allow(dead_code)]
async fn try_connect<F, T, R>(addrs: &[SocketAddr], connect: F) -> IoResult<T>
where
    F: Fn(SocketAddr) -> R,
    F: Send,
    R: Future<Output = IoResult<T>> + Send,
    R::Output: Send,
    T: Send,
{
    let mut last = None;
    for addr in addrs {
        let fut = connect(*addr);
        match fut.await {
            Ok(socket) => return Ok(socket),
            Err(err) => last.replace(err),
        };
    }

    match last {
        Some(last) => Err(last),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "cannot connect with any provided address",
        )),
    }
}
