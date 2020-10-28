//! A collection of useful writers, which allow you to share access to an `Encoder`
//!
//! ## Required/Optional features:
//!
//! | Feature       |                                                                                      |
//! | ------------- | ------------------------------------------------------------------------------------ |
//! | `writer`      | **_required_** for this module                                                       |
//! | `async`       | enables the use of [`asynchronous::Encoder`][async_enc] and [`MpscWriter::shutdown`] |
//! | `sink_stream` | enables the use of [`stream::Encoder`][stream_enc]                                   |
//!
//! You can combine them into configurations such as: `["writer", "async", "sink_stream"]`
//!
//! ## Example
//!
//! ```no_run
//! # use twitchchat::sync::Encoder;
//! # let encoder = Encoder::new(std::io::Cursor::new(Vec::new()));
//! use twitchchat::commands;
//! let writer = twitchchat::writer::MpscWriter::from_encoder(encoder);
//! writer.send(commands::join("#museun")).unwrap();
//! writer.send(commands::part("#museun")).unwrap();
//!
//! // you can clone it
//! let writer2 = writer.clone();
//! writer2.send(commands::part("#museun")).unwrap();
//!
//! // this'll shutdown the receiver.
//! writer.blocking_shutdown();
//!
//! // any futher sends after a shutdown will result in an error
//! writer2.send(commands::raw("foobar\r\n")).unwrap_err();
//! ```
//!
//! [async_enc]: crate::asynchronous::Encoder
//! [stream_enc]: crate::stream::Encoder
//!
use crate::Encodable;

use std::io::Write;

enum Packet {
    Data(Box<[u8]>),
    Quit,
}

/// An MPSC-based writer.
///
/// This type is clonable and is thread-safe.
///
/// This requires `feature = "writer"`
#[derive(Clone)]
pub struct MpscWriter {
    tx: flume::Sender<Packet>,
    wait_for_it: flume::Receiver<()>,
}

impl MpscWriter {
    /// Create a writer from a synchronous [`Encoder`][enc]
    ///
    /// [enc]: crate::sync::Encoder
    pub fn from_encoder<W>(encoder: crate::sync::Encoder<W>) -> Self
    where
        W: Write + Send + 'static,
    {
        let mut encoder = encoder;
        let (tx, rx) = flume::unbounded();
        let (stop_tx, wait_for_it) = flume::bounded(1);
        Self::_start(rx, stop_tx, move |data| encoder.encode(data));
        Self { tx, wait_for_it }
    }

    /// Create a writer from an asynchronous [`Encoder`][enc]
    ///
    /// This requires `feature = "async"`
    ///
    /// [enc]: crate::asynchronous::Encoder
    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub fn from_async_encoder<W>(encoder: crate::asynchronous::Encoder<W>) -> Self
    where
        W: futures_lite::io::AsyncWrite + Send + Unpin + 'static,
    {
        let mut encoder = encoder;
        use futures_lite::future::block_on;
        let (tx, rx) = flume::unbounded();
        let (stop_tx, wait_for_it) = flume::bounded(1);
        Self::_start(rx, stop_tx, move |data| block_on(encoder.encode(data)));
        Self { tx, wait_for_it }
    }

    /// Create a writer from an asynchronous, sink-backed [`Encoder`][enc]
    ///
    /// This requires `feature = "sink_stream"`
    ///
    /// [enc]: crate::stream::Encoder
    #[cfg(feature = "sink_stream")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
    pub fn from_sink_encoder<IO, M>(encoder: crate::stream::Encoder<IO, M>) -> Self
    where
        IO: futures::Sink<M> + Send + Sync + Unpin + 'static,
        M: From<String> + Send + Sync + 'static,
        <IO as futures::Sink<M>>::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut encoder = encoder;
        use futures_lite::future::block_on;
        let (tx, rx) = flume::unbounded();
        let (stop_tx, wait_for_it) = flume::bounded(1);
        Self::_start(rx, stop_tx, move |data| {
            block_on(async { encoder.encode(data).await })
        });
        Self { tx, wait_for_it }
    }

    /// Send this [`Encodable`] item to the writer. This does not block
    ///
    /// This returns an error when:
    /// * the message could not be encoded
    /// * the other side of the MPSC channel hung up
    ///
    /// the other side only hangs up if
    /// * all the senders (the `MpscWriter`) are dropped
    /// * [`MpscWriter::shutdown`] or [`MpscWriter::blocking_shutdown`] are called
    pub fn send(&self, enc: impl Encodable) -> std::io::Result<()> {
        let mut buf = Vec::new();
        enc.encode(&mut buf)?;

        if let Err(flume::TrySendError::Disconnected(..)) =
            self.tx.try_send(Packet::Data(buf.into_boxed_slice()))
        {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        }
        Ok(())
    }

    /// Shutdown the writer (and connection). This blocks the current future
    ///
    /// This sends a `QUIT\r\n` to the connection.
    ///
    /// This requires `feature = "async"`
    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub async fn shutdown(self) {
        let _ = self.send(crate::commands::raw("QUIT\r\n"));
        let _ = self.tx.try_send(Packet::Quit);
        let _ = self.wait_for_it.into_recv_async().await;
    }

    /// Shutdown the writer (and connection). This blocks the current thread
    ///
    /// This sends a `QUIT\r\n` to the connection.
    pub fn blocking_shutdown(self) {
        let _ = self.send(crate::commands::raw("QUIT\r\n"));
        let _ = self.tx.try_send(Packet::Quit);
        let _ = self.wait_for_it.recv();
    }

    fn _start<F>(receiver: flume::Receiver<Packet>, stop: flume::Sender<()>, mut encode: F)
    where
        F: FnMut(Box<[u8]>) -> std::io::Result<()>,
        F: Send + 'static,
    {
        // TODO don't do this for the async one
        let _ = std::thread::spawn(move || {
            let _stop = stop;
            for msg in receiver {
                match msg {
                    Packet::Data(data) => encode(data)?,
                    Packet::Quit => break,
                }
            }
            std::io::Result::Ok(())
        });
    }
}

impl std::fmt::Debug for MpscWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscWriter").finish()
    }
}

#[test]
fn assert_mpscwriter_send_sync() {
    fn assert<T: Send + Sync + 'static>() {}
    fn assert_ref<T: Send + Sync + 'static>(_d: &T) {}
    fn assert_mut_ref<T: Send + Sync + 'static>(_d: &mut T) {}

    assert::<MpscWriter>();

    let (a, _b) = flume::unbounded();
    let (_c, d) = flume::bounded(1);

    let mut w = MpscWriter {
        tx: a,
        wait_for_it: d,
    };

    assert_ref(&w);
    assert_mut_ref(&mut w);
}
