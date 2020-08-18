use super::Channel;
use crate::Encodable;
use std::io::{Result, Write};

/// Grant moderator status to a user.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct GiveMod<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

/// Grant moderator status to a user.
///
/// Use [mods] to list the moderators of this channel.
///
/// [mods]: ./fn.mods.html
pub const fn give_mod<'a>(channel: &'a str, username: &'a str) -> GiveMod<'a> {
    GiveMod { channel, username }
}

impl<'a> Encodable for GiveMod<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/mod {}", self.username)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn give_mod_encode() {
        test_encode(
            give_mod("#museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        );
    }

    #[test]
    fn give_mod_ensure_channel_encode() {
        test_encode(
            give_mod("museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn give_mod_serde() {
        test_serde(
            give_mod("#museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn give_mod_ensure_channel_serde() {
        test_serde(
            give_mod("museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        );
    }
}
