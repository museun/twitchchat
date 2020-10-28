mod error;
pub use error::HandshakeError;

pub mod sync;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod asynchronous;

#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
pub mod stream;
