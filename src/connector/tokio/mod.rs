use crate::BoxedFuture;

mod non_tls;
pub use non_tls::*;

#[cfg(all(feature = "tokio-native-tls", feature = "native-tls"))]
mod native_tls;

#[cfg(all(feature = "tokio-native-tls", feature = "native-tls"))]
pub use self::native_tls::*;

#[cfg(all(feature = "tokio-rustls", feature = "webpki-roots"))]
mod rustls;

#[cfg(all(feature = "tokio-rustls", feature = "webpki-roots"))]
pub use rustls::*;

#[cfg(all(feature = "tokio-openssl", feature = "openssl"))]
mod openssl;
pub use self::openssl::*;
