use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Disables subscribers-only mode.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SubscribersOff<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Disables subscribers-only mode.
pub fn subscribers_off(channel: &str) -> SubscribersOff<'_> {
    let channel = super::make_channel(channel);
    SubscribersOff { channel }
}

impl<'a> Encodable for SubscribersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/subscribersoff"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn subscribers_off_encode() {
        test_encode(
            subscribers_off("#museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        );
    }

    #[test]
    fn subscribers_off_ensure_channel_encode() {
        test_encode(
            subscribers_off("museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_off_serde() {
        test_serde(
            subscribers_off("#museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_off_ensure_channel_serde() {
        test_serde(
            subscribers_off("museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        );
    }
}
