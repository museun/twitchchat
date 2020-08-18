use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Enables emote-only mode (only emoticons may be used in chat).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct EmoteOnly<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Enables emote-only mode (only emoticons may be used in chat).
///
/// Use [emote_only_off] to disable.
///
/// [emote_only_off]: ./fn.emote_only_off.html
pub fn emote_only(channel: &str) -> EmoteOnly<'_> {
    let channel = super::make_channel(channel);
    EmoteOnly { channel }
}

impl<'a> Encodable for EmoteOnly<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/emoteonly"])
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
