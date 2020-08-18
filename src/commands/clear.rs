use std::io::{Result, Write};
use {super::Channel, crate::Encodable};

/// Clear chat history for all users in this room.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Clear<'a> {
    pub(crate) channel: &'a str,
}

/// Clear chat history for all users in this room.
pub const fn clear(channel: &str) -> Clear<'_> {
    Clear { channel }
}

impl<'a> Encodable for Clear<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/clear")
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
