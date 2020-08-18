use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Sends an "emote" message in the third person to the channel
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Me<'a> {
    pub(crate) channel: Cow<'a, str>,
    pub(crate) msg: &'a str,
}

/// Sends an "emote" message in the third person to the channel
pub fn me<'a>(channel: &'a str, msg: &'a str) -> Me<'a> {
    let channel = super::make_channel(channel);
    Me { channel, msg }
}

impl<'a> Encodable for Me<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/me", &self.msg])
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
        test_serde(
            me("museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }
}
