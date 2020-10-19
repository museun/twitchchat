use std::sync::{Arc, Mutex};

pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Determines whether this error is a blocking error
pub fn is_blocking_error(err: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    matches!(err.kind(), WouldBlock | Interrupted | TimedOut)
}

/// Splits this IO object into Read and Write halves
pub fn split<IO>(io: IO) -> (ReadHalf<IO>, WriteHalf<IO>)
where
    IO: std::io::Read + std::io::Write,
{
    let this = Arc::new(Mutex::new(io));
    (ReadHalf(this.clone()), WriteHalf(this))
}

/// Read half of an IO object
pub struct ReadHalf<T>(Arc<Mutex<T>>);

impl<T> std::io::Read for ReadHalf<T>
where
    T: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().read(buf)
    }
}

/// Write half of an IO object
pub struct WriteHalf<T>(Arc<Mutex<T>>);

impl<T> std::io::Write for WriteHalf<T>
where
    T: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}
