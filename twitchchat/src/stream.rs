//! Sink/Stream ([`futures::Sink`] + [`futures::Stream`]) types
//!
//! TODO write a demo here, and more of a description

/// Boxed [`futures::Stream`][ref] trait object
///
/// [ref]: https://docs.rs/futures-core/0.3.7/futures_core/stream/trait.Stream.html
#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
pub type BoxedStream<I> = Box<dyn futures::Stream<Item = I> + Send + Sync + Unpin>;

/// Boxed [`futures::Sink`][ref] trait object
///
/// [ref]: https://docs.rs/futures_sink/0.3.7/futures_sink/trait.Sink.html
#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
pub type BoxedSink<M, E> = Box<dyn futures::Sink<M, Error = E> + Send + Sync + Unpin>;

// re-exports
pub use crate::{
    decoder::{DecodeError, ReadMessage, StreamDecoder},
    encoder::SinkEncoder,
};

pub use crate::handshake::{
    stream::{BoxedDecoder, BoxedEncoder, Decoder, Encoder, Handshake},
    HandshakeError,
};
pub use crate::timeout::{Activity, ActivityReceiver, ActivitySender, TIMEOUT, WINDOW};

pub use crate::identity::{Identity, YourCapabilities};

#[doc(inline)]
pub use crate::asynchronous::idle_detection_loop;

#[doc(inline)]
pub use crate::asynchronous::respond_to_idle_events;
