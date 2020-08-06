use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

/// Cancel the raid.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unraid<'a> {
    pub(crate) channel: &'a str,
}

/// Cancel the raid.
pub const fn unraid(channel: &str) -> Unraid<'_> {
    Unraid { channel }
}

impl<'a> Encodable for Unraid<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unraid"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unraid_encode() {
        test_encode(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unraid_serde() {
        test_serde(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
    }
}
