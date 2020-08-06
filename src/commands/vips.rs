use crate::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

/// Lists the VIPs of this channel.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Vips<'a> {
    pub(crate) channel: &'a str,
}

/// Lists the VIPs of this channel.
pub const fn vips(channel: &str) -> Vips<'_> {
    Vips { channel }
}

impl<'a> Encodable for Vips<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/vips"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn vips_encode() {
        test_encode(vips("#museun"), "PRIVMSG #museun :/vips\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn vips_serde() {
        test_serde(vips("#museun"), "PRIVMSG #museun :/vips\r\n")
    }
}
