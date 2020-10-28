use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Disables subscribers-only mode.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SubscribersOff<'a> {
    pub(crate) channel: &'a str,
}

/// Disables subscribers-only mode.
pub const fn subscribers_off(channel: &str) -> SubscribersOff<'_> {
    SubscribersOff { channel }
}

impl<'a> Encodable for SubscribersOff<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.channel) => "/subscribersoff")
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
