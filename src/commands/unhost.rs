use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Stop hosting another channel.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unhost<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Stop hosting another channel.
pub fn unhost(channel: &str) -> Unhost<'_> {
    let channel = super::make_channel(channel);
    Unhost { channel }
}

impl<'a> Encodable for Unhost<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/unhost"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unhost_encode() {
        test_encode(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n");
    }

    #[test]
    fn unhost_ensure_channel_encode() {
        test_encode(unhost("museun"), "PRIVMSG #museun :/unhost\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unhost_serde() {
        test_serde(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unhost_ensure_channel_serde() {
        test_serde(unhost("museun"), "PRIVMSG #museun :/unhost\r\n")
    }
}
