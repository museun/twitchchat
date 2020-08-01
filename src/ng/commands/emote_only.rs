use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct EmoteOnly<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> EmoteOnly<'a> {
    pub const fn new(channel: &'a str) -> Self {
        Self { channel }
    }
}

pub fn emote_only(channel: &str) -> EmoteOnly<'_> {
    EmoteOnly::new(channel)
}

impl<'a> Encodable for EmoteOnly<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/emoteonly"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn emote_only_encode() {
        test_encode(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn emote_only_serde() {
        test_serde(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n")
    }
}
