use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Lists the moderators of this channel.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Mods<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Lists the moderators of this channel.
pub fn mods(channel: &str) -> Mods<'_> {
    let channel = super::make_channel(channel);
    Mods { channel }
}

impl<'a> Encodable for Mods<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/mods"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn mods_encode() {
        test_encode(mods("#museun"), "PRIVMSG #museun :/mods\r\n");
    }

    #[test]
    fn mods_ensure_channel_encode() {
        test_encode(mods("museun"), "PRIVMSG #museun :/mods\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn mods_serde() {
        test_serde(mods("#museun"), "PRIVMSG #museun :/mods\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn mods_ensure_channel_serde() {
        test_serde(mods("museun"), "PRIVMSG #museun :/mods\r\n");
    }
}
