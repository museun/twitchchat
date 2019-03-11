use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use std::sync::Arc;

/// TestStream is a thread-safe TcpStream-like type thats usable to `mock` the [Client](irc::Client)
// TODO write usage
#[derive(Clone)]
pub struct TestStream {
    read_rx: Receiver<Vec<u8>>,
    write_tx: Sender<Vec<u8>>,

    read_tx: Sender<Vec<u8>>,
    write_rx: Receiver<Vec<u8>>,

    buf: Arc<RwLock<Vec<u8>>>,
}

impl Default for TestStream {
    fn default() -> Self {
        let (read_tx, read_rx) = bounded(1);
        let (write_tx, write_rx) = bounded(1);

        Self {
            read_rx,
            write_tx,

            read_tx,
            write_rx,

            buf: Arc::new(RwLock::new(vec![])),
        }
    }
}

impl TestStream {
    /// Create a new TestStream
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads a line from the stream (e.g. read what has written to the client)
    /// **NOTE** Keeps the trailing \r\n
    pub fn read_message(&mut self) -> Option<String> {
        self.write_rx
            .recv()
            .ok()
            .and_then(|d| String::from_utf8(d).ok())
    }

    /// Writes a line to the stream (e.g. what should be read from the client)
    pub fn write_message<S: AsRef<[u8]>>(&mut self, data: S) -> Option<()> {
        let data = data.as_ref().to_vec();
        self.read_tx.send(data).ok()?;
        Some(())
    }
}

impl std::io::Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use crossbeam_channel::TryRecvError;
        use std::io::{Cursor, Error, ErrorKind};

        match self.read_rx.try_recv() {
            Ok(data) => Cursor::new(data).read(buf), // shitty
            Err(TryRecvError::Disconnected) => Err(Error::new(ErrorKind::NotConnected, "")),
            Err(TryRecvError::Empty) => Err(Error::new(ErrorKind::WouldBlock, "")),
        }
    }
}

impl std::io::Write for TestStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf == b"\r\n" {
            return Ok(buf.len());
        }

        self.buf.write().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.buf.read().is_empty() {
            return Ok(());
        }

        use crossbeam_channel::TrySendError;
        use std::io::{Error, ErrorKind};

        let buf = {
            let mut buf = self.buf.write();
            buf.drain(..).collect()
        };

        match self.write_tx.try_send(buf) {
            Ok(_) => Ok(()),
            Err(TrySendError::Disconnected(..)) => Err(Error::new(ErrorKind::NotConnected, "")),
            Err(TrySendError::Full(..)) => Err(Error::new(ErrorKind::WouldBlock, "")),
        }
    }
}
