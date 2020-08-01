use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Ping<'a> {
    pub(crate) token: &'a str,
}

impl<'a> Ping<'a> {
    pub const fn new(token: &'a str) -> Self {
        Self { token }
    }
}

pub fn ping(token: &str) -> Ping<'_> {
    Ping::new(token)
}

impl<'a> Encodable for Ping<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts(&[&"PING", &self.token])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn ping_encode() {
        test_encode(ping("123456789"), "PING 123456789\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn ping_serde() {
        test_serde(ping("123456789"), "PING 123456789\r\n");
    }
}
