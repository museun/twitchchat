use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Revoke moderator status from a user.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unmod<'a> {
    pub(crate) channel: Cow<'a, str>,
    pub(crate) username: &'a str,
}

/// Revoke moderator status from a user.
///
/// Use [mods] to list the moderators of this channel.
///
/// [mods]: ./fn.mods.html
pub fn unmod<'a>(channel: &'a str, username: &'a str) -> Unmod<'a> {
    let channel = super::make_channel(channel);
    Unmod { channel, username }
}

impl<'a> Encodable for Unmod<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/unmod", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unmod_encode() {
        test_encode(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }

    #[test]
    fn unmod_ensure_channel_encode() {
        test_encode(
            unmod("museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unmod_serde() {
        test_serde(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unmod_ensure_channel_serde() {
        test_serde(
            unmod("museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }
}
