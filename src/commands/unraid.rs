use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Cancel the raid.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unraid<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Cancel the raid.
pub fn unraid(channel: &str) -> Unraid<'_> {
    let channel = super::make_channel(channel);
    Unraid { channel }
}

impl<'a> Encodable for Unraid<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/unraid"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unraid_encode() {
        test_encode(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n");
        test_encode(unraid("museun"), "PRIVMSG #museun :/unraid\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unraid_serde() {
        test_serde(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n");
        test_serde(unraid("museun"), "PRIVMSG #museun :/unraid\r\n")
    }
}
