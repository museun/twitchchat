use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Disables emote-only mode.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct EmoteOnlyOff<'a> {
    pub(crate) channel: &'a str,
}

/// Disables emote-only mode.
pub const fn emote_only_off(channel: &str) -> EmoteOnlyOff<'_> {
    EmoteOnlyOff { channel }
}

impl<'a> Encodable for EmoteOnlyOff<'a> {
fn encode(&self, buf:&mut dyn Write) -> Result<()>
    {
        write_cmd!(buf, Channel(self.channel) => "/emoteonlyoff")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn emote_only_off_encode() {
        test_encode(
            emote_only_off("#museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        );
    }

    #[test]
    fn emote_only_off_ensure_channel_encode() {
        test_encode(
            emote_only_off("museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn emote_only_off_serde() {
        test_serde(
            emote_only_off("#museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn emote_only_off_ensure_channel_serde() {
        test_serde(
            emote_only_off("museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        );
    }
}
