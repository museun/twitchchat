use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Clear chat history for all users in this room.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Clear<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Clear chat history for all users in this room.
pub fn clear(channel: &str) -> Clear<'_> {
    let channel = super::make_channel(channel);
    Clear { channel }
}

impl<'a> Encodable for Clear<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/clear"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn clear_encode() {
        test_encode(clear("#museun"), "PRIVMSG #museun :/clear\r\n");
    }

    #[test]
    fn clear_ensure_channel_encode() {
        test_encode(clear("museun"), "PRIVMSG #museun :/clear\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn clear_serde() {
        test_serde(clear("#museun"), "PRIVMSG #museun :/clear\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn clear_ensure_channel_serde() {
        test_serde(clear("museun"), "PRIVMSG #museun :/clear\r\n")
    }
}
