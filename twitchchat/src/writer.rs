//! A collection of useful writers, which allow you to share access to an [`Encoder`]/[`AsyncEncoder`]
//!
//! ## Required/Optional features:
//!
//! | Feature        |                                                                  |
//! | -------------- | ---------------------------------------------------------------- |
//! | `writer`       | **_required_** for this module                                   |
//! | `async`        | enables the use of [`AsyncEncoder`] and [`MpscWriter::shutdown`] |
//! | `async` + `ws` | enables the use of [`WsEncoder`] and [`MpscWriter::shutdown`]    |
//!
//! ## Example
//!
//! ```no_run
//! # use twitchchat::Encoder;
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
//! writer.shutdown_sync();
//!
//! // any futher sends after a shutdown will result in an error
//! writer2.send(commands::raw("foobar\r\n")).unwrap_err();
//! ```
use crate::*;

use std::io::Write;
cfg_async! {
    use futures_lite::io::AsyncWrite;
    cfg_ws! {
        use futures_lite::io::AsyncRead;
        use crate::ws::WsEncoder;
    }
}

enum Packet {
    Data(Box<[u8]>),
    Quit,
}

/// An MPSC-based writer.
///
/// This type is clonable and is thread-safe.
#[derive(Clone)]
pub struct MpscWriter {
    tx: flume::Sender<Packet>,
    wait_for_it: flume::Receiver<()>,
}

impl std::fmt::Debug for MpscWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscWriter").finish()
    }
}

impl MpscWriter {
    /// Create a writer from a synchronous [`Encoder`]
    pub fn from_encoder<W>(encoder: Encoder<W>) -> Self
    where
        W: Write + Send + 'static,
    {
        let mut encoder = encoder;
        let (tx, rx) = flume::unbounded();
        let (stop_tx, wait_for_it) = flume::bounded(1);
        Self::_start(rx, stop_tx, move |data| encoder.encode(data));
        Self { tx, wait_for_it }
    }

    cfg_async! {
    /// Create a writer from an asynchronous [`AsyncEncoder`]
    pub fn from_async_encoder<W>(encoder: AsyncEncoder<W>) -> Self
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let mut encoder = encoder;
        use futures_lite::future::block_on;
        let (tx, rx) = flume::unbounded();
        let (stop_tx, wait_for_it) = flume::bounded(1);
        Self::_start(rx, stop_tx, move |data| block_on(encoder.encode(data)));
        Self { tx, wait_for_it }
    }
    }

    cfg_async! {
    cfg_ws!{
    /// Create a writer from an asynchronous [`WsEncoder`]
    pub fn from_ws_encoder<W>(encoder: WsEncoder<W>) -> Self
    where
        W: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let mut encoder = encoder;
        use futures_lite::future::block_on;
        let (tx, rx) = flume::unbounded();
        let (stop_tx, wait_for_it) = flume::bounded(1);
        Self::_start(rx, stop_tx, move |data| {
            block_on(async {
                encoder
                    .encode(data)
                    .await
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            })
        });
        Self { tx, wait_for_it }
    }
    }
    }

    /// Send this [`Encodable`] item to the writer. This does not block
    ///
    /// This returns an error when:
    /// * the message could not be encoded
    /// * the other side of the MPSC channel hung up
    ///
    /// the other side only hangs up if
    /// * all the senders (the `MpscWriter`) are dropped
    /// * [`MpscWriter::shutdown`] or [`MpscWriter::shutdown_sync`] are called
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

    cfg_async! {
    /// Shutdown the writer (and connection). This blocks the current future
    ///
    /// This sends a `QUIT\r\n` to the connection.
    pub async fn shutdown(self) {
        let _ = self.send(crate::commands::raw("QUIT\r\n"));
        let _ = self.tx.try_send(Packet::Quit);
        let _ = self.wait_for_it.into_recv_async().await;
    }
    }

    /// Shutdown the writer (and connection). This blocks the current thread
    ///
    /// This sends a `QUIT\r\n` to the connection.
    pub fn shutdown_sync(self) {
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
