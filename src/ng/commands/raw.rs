use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Raw<'a> {
    pub data: &'a str,
}

pub fn raw(data: &str) -> Raw<'_> {
    Raw { data }
}

impl<'a> Encodable for Raw<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).write_bytes(self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn raw_encode() {
        test_encode(
            raw("PRIVMSG #test :this is a test"),
            "PRIVMSG #test :this is a test\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn raw_serde() {
        test_serde(
            raw("PRIVMSG #test :this is a test"),
            "PRIVMSG #test :this is a test\r\n",
        );
    }
}
