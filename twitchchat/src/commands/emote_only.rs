use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Enables emote-only mode (only emoticons may be used in chat).
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct EmoteOnly<'a> {
    pub(crate) channel: &'a str,
}

/// Enables emote-only mode (only emoticons may be used in chat).
///
/// Use [emote_only_off] to disable.
///
/// [emote_only_off]: super::emote_only_off()
pub const fn emote_only(channel: &str) -> EmoteOnly<'_> {
    EmoteOnly { channel }
}

impl<'a> Encodable for EmoteOnly<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.channel) => "/emoteonly")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn emote_only_encode() {
        test_encode(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n");
    }

    #[test]
    fn emote_only_ensure_channel_encode() {
        test_encode(emote_only("museun"), "PRIVMSG #museun :/emoteonly\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn emote_only_serde() {
        test_serde(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn emote_only_ensure_channel_serde() {
        test_serde(emote_only("museun"), "PRIVMSG #museun :/emoteonly\r\n")
    }
}
