use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unhost<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> Unhost<'a> {
    pub const fn new(channel: &'a str) -> Self {
        Self { channel }
    }
}

pub fn unhost(channel: &str) -> Unhost<'_> {
    Unhost::new(channel)
}

impl<'a> Encodable for Unhost<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unhost"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unhost_encode() {
        test_encode(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unhost_serde() {
        test_serde(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
    }
}
