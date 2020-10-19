use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Lists the moderators of this channel.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Mods<'a> {
    pub(crate) channel: &'a str,
}

/// Lists the moderators of this channel.
pub const fn mods(channel: &str) -> Mods<'_> {
    Mods { channel }
}

impl<'a> Encodable for Mods<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.channel) => "/mods")
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
