use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Disables followers-only mode.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct FollowersOff<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Disables followers-only mode.
pub fn followers_off(channel: &str) -> FollowersOff<'_> {
    let channel = super::make_channel(channel);
    FollowersOff { channel }
}

impl<'a> Encodable for FollowersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/followersoff"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn followers_off_encode() {
        test_encode(
            followers_off("#museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        );
    }

    #[test]
    fn followers_off_ensure_channel_encode() {
        test_encode(
            followers_off("museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn followers_off_serde() {
        test_serde(
            followers_off("#museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn followers_off_ensure_channel_serde() {
        test_serde(
            followers_off("museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        );
    }
}
