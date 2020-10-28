//! This is a collection of type aliases to make naming things easier
//!
//! Additionally, two concrete types are provided for when using `std::io` types.
//!
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

/// A boxed [`std::io::Read`] trait object
pub type BoxedRead = Box<dyn Read + Send + Sync>;
/// A boxed [`std::io::Write`] trait object
pub type BoxedWrite = Box<dyn Write + Send + Sync>;

#[derive(Debug, Clone)]
/// Read half of an `std::io::Read + std::io::Write` implementation
pub struct ReadHalf<T>(Arc<Mutex<T>>);

impl<T> Read for ReadHalf<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().read(buf)
    }
}

#[derive(Debug, Clone)]
/// Write half of an `std::io::Read + std::io::Write` implementation
pub struct WriteHalf<T>(Arc<Mutex<T>>);

impl<T> Write for WriteHalf<T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

/// Splits this IO object into `Read` and `Write` halves
#[allow(dead_code)]
pub(crate) fn split<IO>(io: IO) -> (ReadHalf<IO>, WriteHalf<IO>)
where
    IO: Read + Write,
{
    let this = Arc::new(Mutex::new(io));
    (ReadHalf(this.clone()), WriteHalf(this))
}
