use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Lists the commands available to you in this room.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Help<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Lists the commands available to you in this room.
pub fn help(channel: &str) -> Help<'_> {
    let channel = super::make_channel(channel);
    Help { channel }
}

impl<'a> Encodable for Help<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/help"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn help_encode() {
        test_encode(help("#museun"), "PRIVMSG #museun :/help\r\n");
    }

    #[test]
    fn help_ensure_channel_encode() {
        test_encode(help("museun"), "PRIVMSG #museun :/help\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn help_serde() {
        test_serde(help("#museun"), "PRIVMSG #museun :/help\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn help_ensure_channel_serde() {
        test_serde(help("museun"), "PRIVMSG #museun :/help\r\n");
    }
}
