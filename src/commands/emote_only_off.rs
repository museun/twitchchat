use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Disables emote-only mode.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct EmoteOnlyOff<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Disables emote-only mode.
pub fn emote_only_off(channel: &str) -> EmoteOnlyOff<'_> {
    let channel = super::make_channel(channel);
    EmoteOnlyOff { channel }
}

impl<'a> Encodable for EmoteOnlyOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/emoteonlyoff"])
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
        test_serde(
            emote_only_off("museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        );
    }
}
