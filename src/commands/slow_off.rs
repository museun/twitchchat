use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Disables slow mode.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SlowOff<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Disables slow mode.
pub fn slow_off(channel: &str) -> SlowOff<'_> {
    let channel = super::make_channel(channel);
    SlowOff { channel }
}

impl<'a> Encodable for SlowOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/slowoff"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn slow_off_encode() {
        test_encode(slow_off("#museun"), "PRIVMSG #museun :/slowoff\r\n");
    }

    #[test]
    fn slow_off_ensure_channel_encode() {
        test_encode(slow_off("museun"), "PRIVMSG #museun :/slowoff\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn slow_off_serde() {
        test_serde(slow_off("#museun"), "PRIVMSG #museun :/slowoff\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn slow_off_ensure_channel_serde() {
        test_serde(slow_off("museun"), "PRIVMSG #museun :/slowoff\r\n");
    }
}
