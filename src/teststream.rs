use parking_lot::RwLock;
use std::sync::Arc;

/// TestStream is a thread-safe TcpStream-like type thats usable to `mock` the [Client](irc::Client)
// TODO write usage
// TODO get rid of the drain().collect()
#[derive(Default, Clone)]
pub struct TestStream {
    read: Arc<RwLock<Vec<u8>>>,
    write: Arc<RwLock<Vec<u8>>>,
    buf: Arc<RwLock<Vec<u8>>>,
}

impl TestStream {
    /// Create a new TestStream
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads a line from the stream (e.g. read what has written to the client)
    /// **NOTE** Keeps the trailing \r\n
    pub fn read_message(&mut self) -> Option<String> {
        let w = self.write.write().drain(..).collect::<Vec<_>>();
        String::from_utf8(w).ok()
    }

    /// Writes a line to the stream (e.g. what should be read from the client)
    pub fn write_message<S: AsRef<[u8]>>(&mut self, data: S) {
        let mut w = self.read.write();
        w.extend_from_slice(&data.as_ref());
        w.extend_from_slice(&[b'\r', b'\n']);
    }
}

impl std::io::Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::{Cursor, Error, ErrorKind};

        if self.read.read().ends_with(&[b'\r', b'\n']) {
            let w = self.read.write().drain(..).collect::<Vec<_>>();
            return Cursor::new(w).read(buf);
        }

        Err(Error::new(ErrorKind::WouldBlock, ""))
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
        use std::io::{copy, Cursor};
        if self.buf.read().is_empty() {
            return Ok(());
        }

        let w = self.buf.write().drain(..).collect::<Vec<_>>();
        copy(&mut Cursor::new(w), &mut *self.write.write()).map(|_| ())
    }
}
