use crate::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

/// Stop hosting another channel.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unhost<'a> {
    pub(crate) channel: &'a str,
}

/// Stop hosting another channel.
pub const fn unhost(channel: &str) -> Unhost<'_> {
    Unhost { channel }
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
