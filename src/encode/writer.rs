#[allow(dead_code)]
pub mod channel {
    #[cfg(feature = "crossbeam-channel")]
    mod inner {
        pub type Tx<T> = crossbeam_channel::Sender<T>;
        pub type Rx<T> = crossbeam_channel::Receiver<T>;
        pub use crossbeam_channel::SendError;
    }

    #[cfg(not(feature = "crossbeam-channel"))]
    mod inner {
        pub type Tx<T> = std::sync::mpsc::Sender<T>;
        pub type Rx<T> = std::sync::mpsc::Receiver<T>;
        pub use std::sync::mpsc::SendError;
    }

    pub use inner::*;

    #[cfg(feature = "crossbeam-channel")]
    /// Create an unbounded channel
    pub fn channel<T>() -> (Tx<T>, Rx<T>) {
        crossbeam_channel::unbounded()
    }

    #[cfg(not(feature = "crossbeam-channel"))]
    /// Create an unbounded channel
    pub fn channel<T>() -> (Tx<T>, Rx<T>) {
        std::sync::mpsc::channel()
    }
}

use channel::*;
use std::io::{Error, ErrorKind, Write};

/// A writer that allows sending messages to the client
pub type SyncWriter = crate::Encoder<SyncMpscWriter>;

/// A channel-based synchronous writer
pub struct SyncMpscWriter {
    tx: Tx<Vec<u8>>,
    buf: Vec<u8>,
}

impl std::fmt::Debug for SyncMpscWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncMpscWriter").finish()
    }
}

impl Clone for SyncMpscWriter {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            buf: Vec::new(),
        }
    }
}

impl SyncMpscWriter {
    /// Create a new SyncMpscWriter from this channel's sender
    pub fn new(tx: Tx<Vec<u8>>) -> Self {
        Self {
            tx,
            buf: Vec::new(),
        }
    }
}

impl Write for SyncMpscWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // if the receiver has been dropped, then we should return an error.
        self.tx
            .send(std::mem::take(&mut self.buf))
            .map_err(|_| Error::new(ErrorKind::UnexpectedEof, "cannot write, other half dropped"))
    }
}
