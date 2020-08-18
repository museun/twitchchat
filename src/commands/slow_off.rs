use super::Channel;
use crate::Encodable;
use std::io::{Result, Write};

/// Disables slow mode.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SlowOff<'a> {
    pub(crate) channel: &'a str,
}

/// Disables slow mode.
pub const fn slow_off(channel: &str) -> SlowOff<'_> {
    SlowOff { channel }
}

impl<'a> Encodable for SlowOff<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(&self.channel) => "/slowoff")
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
