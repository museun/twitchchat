use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Sends an "emote" message in the third person to the channel
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Me<'a> {
    pub(crate) channel: &'a str,
    pub(crate) msg: &'a str,
}

/// Sends an "emote" message in the third person to the channel
pub const fn me<'a>(channel: &'a str, msg: &'a str) -> Me<'a> {
    Me { channel, msg }
}

impl<'a> Encodable for Me<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.channel) => "/me {}", self.msg)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn me_encode() {
        test_encode(
            me("#museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }

    #[test]
    fn me_ensure_channel_encode() {
        test_encode(
            me("museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn me_serde() {
        test_serde(
            me("#museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn me_ensure_channel_serde() {
        test_serde(
            me("museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }
}
