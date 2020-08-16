//! This module lets you choose which runtime you want to use.
//!
//! By default, TLS is disabled to make building the crate on various platforms easier.
//!
//! To use..
//!
//! | Read/Write provider                                        | Features                |
//! | ---                                                        | ---                     |
//! | [`async_io`](https://docs.rs/async-io/latest/async_io/)    |`async-io`               |
//! | [`smol`](https://docs.rs/smol/latest/smol/)                |`smol`                   |
//! | [`async_std`](https://docs.rs/async-std/latest/async_std/) |`async-std`              |
//! | [`tokio`](https://docs.rs/tokio/latest/tokio/)             |`tokio` and `tokio-util` |
//!
//! ## TLS
//!
//! If you want TLS supports, this crate currently supports using various [`rustls`](https://docs.rs/rustls/latest/rustls/) wrappers.
//!
//! Enable the above runtime and also enable the cooresponding features:
//!
//! | Read/Write provider                                        | Runtime     | Features                                        |
//! | ---                                                        | ---         | ---                                             |
//! | [`async_io`](https://docs.rs/async-io/latest/async_io/)    | `async_io`  | `async-tls`                                     |
//! | [`smol`](https://docs.rs/smol/latest/smol/)                | `smol`      | `async-tls`                                     |
//! | [`async_std`](https://docs.rs/async-std/latest/async_std/) | `async_std` | `async-tls`                                     |
//! | [`tokio`](https://docs.rs/tokio/latest/tokio/)             | `tokio`     | `tokio-util`, `tokio-rustls` and `webpki-roots` |
use futures_lite::{AsyncRead, AsyncWrite};
use std::{future::Future, io::Result as IoResult, net::SocketAddr};

#[cfg(feature = "async-io")]
/// Connector for using an [`async_io`](https://docs.rs/async-io/latest/async_io/) wrapper over [`std::net::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
pub mod async_io;

#[cfg(feature = "async-std")]
/// Connector for using an [`async_std::net::TcpStream`](https://docs.rs/async-std/latest/async_std/net/struct.TcpStream.html)
pub mod async_std;

#[cfg(feature = "smol")]
/// Connector for using a [`smol::Async`](https://docs.rs/smol/latest/smol/struct.Async.html) wrapper over [`std::net::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
pub mod smol;

#[cfg(all(feature = "tokio", feature = "tokio-util"))]
/// Connector for using a [`tokio::net::TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html)
pub mod tokio;

/// The connector trait. This is used to abstract out runtimes.
///
/// You can implement this on your own type to provide a custom connection behavior.
pub trait Connector: Send + Sync + Clone {
    /// Output IO type returned by calling `connect`
    ///
    /// This type must implement `futures::io::AsyncRead` and `futures::io::AsyncWrite`
    type Output: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static;
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
    F: Fn(SocketAddr) -> R + Send,
    R: Future<Output = IoResult<T>> + Send,
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

mod required {
    #[cfg(all(
        feature = "async-tls",
        not(any(feature = "async-io", feature = "async-std", feature = "smol"))
    ))]
    compile_error! {
        "'async-io' or 'async-std' or 'smol' must be enabled when 'async-tls' is enabled"
    }

    #[cfg(all(feature = "tokio", not(feature = "tokio-util")))]
    compile_error! {
        "'tokio-util' must be enabled when 'tokio' is enabled"
    }

    #[cfg(all(
        feature = "tokio-rustls",
        not(all(feature = "tokio", feature = "tokio-util", feature = "webpki-roots"))
    ))]
    compile_error! {
        "'tokio', 'tokio-util' and 'webpki-roots' must be enabled when 'tokio-rustls' is enabled"
    }
}
