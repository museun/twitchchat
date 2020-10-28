use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Cancel the raid.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unraid<'a> {
    pub(crate) channel: &'a str,
}

/// Cancel the raid.
pub const fn unraid(channel: &str) -> Unraid<'_> {
    Unraid { channel }
}

impl<'a> Encodable for Unraid<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.channel) => "/unraid")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unraid_encode() {
        test_encode(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n");
    }

    #[test]
    fn unraid_ensure_channel_encode() {
        test_encode(unraid("museun"), "PRIVMSG #museun :/unraid\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unraid_serde() {
        test_serde(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unraid_ensure_channel_serde() {
        test_serde(unraid("museun"), "PRIVMSG #museun :/unraid\r\n")
    }
}
