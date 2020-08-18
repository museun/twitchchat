use super::ByteWriter;
use crate::Encodable;
use std::io::{Result, Write};

/// Respond to a server request (normally a PING) with the provided token
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Pong<'a> {
    pub(crate) token: &'a str,
}

/// Respond to a server request (normally a PING) with the provided token
pub const fn pong(token: &str) -> Pong<'_> {
    Pong { token }
}

impl<'a> Encodable for Pong<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts_term(&[&"PONG", &" :", &self.token])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn pong_encode() {
        test_encode(pong("123456789"), "PONG :123456789\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn pong_serde() {
        test_serde(pong("123456789"), "PONG :123456789\r\n");
    }
}
