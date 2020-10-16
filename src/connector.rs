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
//! | [`tokio`](https://docs.rs/tokio/0.2/tokio/)                |`tokio` and `tokio-util` |
//!
//! ## TLS
//!
//! If you want TLS supports, enable the above runtime and also enable the cooresponding features:
//!
//! | Read/Write provider                                        | Runtime     | Features                                             | TLS backend                |
//! | ---------------------------------------------------------- | ----------- | ---------------------------------------------------- | -------------------------- |
//! | [`async_io`](https://docs.rs/async-io/latest/async_io/)    | `async_io`  | `"async-tls"`                                        | [`rustls`][rustls]         |
//! | [`smol`](https://docs.rs/smol/latest/smol/)                | `smol`      | `"async-tls"`                                        | [`rustls`][rustls]         |
//! | [`async_std`](https://docs.rs/async-std/latest/async_std/) | `async_std` | `"async-tls"`                                        | [`rustls`][rustls]         |
//! | [`tokio`](https://docs.rs/tokio/0.2/tokio/)                | `tokio`     | `"tokio-util"`, `"tokio-rustls"`, `"webpki-roots"`   | [`rustls`][rustls]         |
//! | [`tokio`](https://docs.rs/tokio/0.2/tokio/)                | `tokio`     | `"tokio-util"`, `"tokio-native-tls"`, `"native-tls"` | [`native-tls`][native-tls] |
//! | [`tokio`](https://docs.rs/tokio/0.2/tokio/)                | `tokio`     | `"tokio-util"`, `"tokio-openssl"`, `"openssl"`       | [`openssl`][openssl]       |
//!
//! [rustls]: https://docs.rs/rustls/0.18.1/rustls/
//! [native-tls]: https://docs.rs/native-tls/0.2.4/native_tls/
//! [openssl]: https://docs.rs/openssl/0.10/openssl/
//!
use futures_lite::{AsyncRead, AsyncWrite};
use std::{future::Future, io::Result as IoResult, net::SocketAddr};

#[allow(unused_macros)]
macro_rules! connector_ctor {
    (non_tls: $(#[$meta:meta])*) => {
        #[doc = "Create a new"]
        $(#[$meta])*
        #[doc = "non-TLS connector that connects to the ***default Twitch*** address."]
        pub fn twitch() -> ::std::io::Result<Self> {
            Self::custom($crate::TWITCH_IRC_ADDRESS)
        }

        #[doc = "Create a new"]
        $(#[$meta])*
        #[doc = "non-TLS connector with a custom address."]
        pub fn custom<A>(addrs: A) -> ::std::io::Result<Self>
        where
            A: ::std::net::ToSocketAddrs,
        {
            addrs.to_socket_addrs().map(|addrs| Self {
                addrs: addrs.collect(),
            })
        }
    };

    (tls: $(#[$meta:meta])*) => {
        #[doc = "Create a new"]
        $(#[$meta])*
        #[doc = "TLS connector that connects to the ***default Twitch*** address."]
        pub fn twitch() -> ::std::io::Result<Self> {
            Self::custom($crate::TWITCH_IRC_ADDRESS_TLS, $crate::TWITCH_TLS_DOMAIN)
        }


        #[doc = "Create a new"]
        $(#[$meta])*
        #[doc = "TLS connector with a custom address and TLS domain."]
        pub fn custom<A, D>(addrs: A, domain: D) -> ::std::io::Result<Self>
        where
            A: ::std::net::ToSocketAddrs,
            D: Into<::std::string::String>,
        {
            let tls_domain = domain.into();
            addrs.to_socket_addrs().map(|addrs| Self {
                addrs: addrs.collect(),
                tls_domain,
            })
        }
    };
}

#[cfg(feature = "async-io")]
/// Connector for using an [`async_io`](https://docs.rs/async-io/latest/async_io/) wrapper over [`std::net::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
pub mod async_io;

#[cfg(feature = "async-io")]
#[doc(inline)]
pub use self::async_io::Connector as AsyncIoConnector;

#[cfg(all(feature = "async-io", feature = "async-tls"))]
#[doc(inline)]
pub use self::async_io::ConnectorTls as AsyncIoConnectorTls;

#[cfg(feature = "async-std")]
/// Connector for using an [`async_std::net::TcpStream`](https://docs.rs/async-std/latest/async_std/net/struct.TcpStream.html)
pub mod async_std;

#[cfg(feature = "async-std")]
#[doc(inline)]
pub use self::async_std::Connector as AsyncStdConnector;

#[cfg(all(feature = "async-std", feature = "async-tls"))]
#[doc(inline)]
pub use self::async_std::ConnectorTls as AsyncStdConnectorTls;

#[cfg(feature = "smol")]
/// Connector for using a [`smol::Async`](https://docs.rs/smol/latest/smol/struct.Async.html) wrapper over [`std::net::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
pub mod smol;

#[cfg(feature = "smol")]
#[doc(inline)]
pub use self::smol::Connector as SmolConnector;

#[cfg(all(feature = "smol", feature = "async-tls"))]
#[doc(inline)]
pub use self::smol::ConnectorTls as SmolConnectorTls;

#[cfg(all(feature = "tokio", feature = "tokio-util"))]
/// Connector for using a [`tokio::net::TcpStream`](https://docs.rs/tokio/0.2/tokio/net/struct.TcpStream.html)
pub mod tokio;

#[cfg(all(feature = "tokio", feature = "tokio-util"))]
#[doc(inline)]
pub use self::tokio::Connector as TokioConnector;

#[cfg(all(
    feature = "tokio",
    feature = "tokio-util",
    feature = "tokio-rustls",
    feature = "webpki-roots"
))]
#[doc(inline)]
pub use self::tokio::ConnectorRustTls as TokioConnectorRustTls;

#[cfg(all(
    feature = "tokio",
    feature = "tokio-util",
    feature = "tokio-native-tls",
    feature = "native-tls"
))]
#[doc(inline)]
pub use self::tokio::ConnectorNativeTls as TokioConnectorNativeTls;

#[cfg(all(
    feature = "tokio",
    feature = "tokio-util",
    feature = "tokio-openssl",
    feature = "openssl"
))]
#[doc(inline)]
pub use self::tokio::ConnectorOpenSsl as TokioConnectorOpenSsl;

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
    /// e.g. `Box::pin(async move { std::net::TcpStream::connect("someaddr") })`
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
        feature = "tokio-native-tls",
        not(all(feature = "tokio", feature = "tokio-util", feature = "native-tls"))
    ))]
    compile_error! {
        "'tokio', 'tokio-util' and 'native-tls' must be enabled when 'tokio-native-tls' is enabled"
    }

    #[cfg(all(
        feature = "tokio-rustls",
        not(all(feature = "tokio", feature = "tokio-util", feature = "webpki-roots"))
    ))]
    compile_error! {
        "'tokio', 'tokio-util' and 'webpki-roots' must be enabled when 'tokio-rustls' is enabled"
    }

    #[cfg(all(
        feature = "tokio-openssl",
        not(all(feature = "tokio", feature = "tokio-util", feature = "openssl"))
    ))]
    compile_error! {
        "'tokio', 'tokio-util' and 'openssl' must be enabled when 'tokio-openssl' is enabled"
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod testing {
    use crate::connector::Connector as ConnectorTrait;
    use futures_lite::{AsyncRead, AsyncWrite};

    pub fn assert_connector<T: ConnectorTrait>() {}
    pub fn assert_type_is_read_write<T: AsyncRead + AsyncWrite>() {}
    pub fn assert_obj_is_sane<T>(_obj: T)
    where
        T: ConnectorTrait,
        T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
        for<'a> &'a T::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
    {
    }
}
