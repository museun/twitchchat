use super::*;

/// A Raw message. Used to send a raw message
#[derive(Copy, Clone, Debug)]
pub struct Raw<'a> {
    pub(crate) raw: &'a str,
}

impl<'a> Encodable for Raw<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.raw.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// Send a raw message
pub fn raw(raw: &str) -> Raw<'_> {
    Raw { raw }
}
