use std::{
    io::{Result as IoResult, Write},
    rc::Rc,
    sync::Arc,
};

/// A trait to allow writing messags to any [std::io::Write] implementation
pub trait Encodable {
    /// Encode this message to the provided [std::io::Write] implementation
    fn encode(&self, buf: &mut dyn Write) -> IoResult<()>;
}

impl<T> Encodable for &T
where
    T: Encodable + ?Sized,
{
    fn encode(&self, buf: &mut dyn Write) -> IoResult<()> {
        <_ as Encodable>::encode(*self, buf)
    }
}

impl Encodable for str {
    fn encode(&self, buf: &mut dyn Write) -> IoResult<()> {
        buf.write_all(self.as_bytes())
    }
}

impl Encodable for String {
    fn encode(&self, buf: &mut dyn Write) -> IoResult<()> {
        buf.write_all(self.as_bytes())
    }
}

macro_rules! encodable_byte_slice {
    ($($ty:ty)*) => {
        $(impl Encodable for $ty {
            fn encode(&self, buf: &mut dyn Write) -> IoResult<()> {
                buf.write_all(self)
            }
        })*
    };
}

encodable_byte_slice! {
    [u8]
    Box<[u8]>
    Rc<[u8]>
    Arc<[u8]>
    Vec<u8>
}
