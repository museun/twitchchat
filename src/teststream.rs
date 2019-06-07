use std::sync::{Arc, Mutex};

/// TestStream is a thread-safe TcpStream-like type thats usable to `mock` the [Client](../struct.CLient.html)
// TODO write usage
// TODO get rid of the drain().collect()
#[derive(Default, Clone)]
pub struct TestStream {
    read: Arc<Mutex<Vec<u8>>>,
    write: Arc<Mutex<Vec<u8>>>,
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TestStream {
    /// Create a new TestStream
    pub fn new() -> Self {
        Self::default()
    }

    /// Drains the internal from the stream (e.g. read what has written to the client)
    ///
    /// **NOTE** Keeps the trailing \r\n
    pub fn drain_buffer(&mut self) -> String {
        let mut w = self.write.lock().unwrap();
        if w.is_empty() {
            return "".into();
        }
        let v = w.drain(..).collect::<Vec<_>>();
        String::from_utf8_lossy(&v).to_string()
    }

    /// Writes a line to the stream (e.g. what should be read from the client)
    pub fn write_message<S: AsRef<[u8]>>(&mut self, data: S) {
        let mut w = self.read.lock().unwrap();
        w.extend_from_slice(&data.as_ref());
        w.extend_from_slice(&[b'\r', b'\n']);
    }
}

impl std::io::Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::{Cursor, Error, ErrorKind};

        let mut read = self.read.lock().unwrap();
        if read.ends_with(&[b'\r', b'\n']) {
            let w = read.drain(..).collect::<Vec<_>>();
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
        self.buf.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut buf = self.buf.lock().unwrap();
        if buf.is_empty() {
            return Ok(());
        }
        self.write.lock().unwrap().extend(buf.drain(..));
        Ok(())
    }
}
