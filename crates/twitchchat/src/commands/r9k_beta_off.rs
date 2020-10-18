use super::{Channel, Encodable};

use std::io::{Result, Write};

/// Disables r9k mode.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct R9kBetaOff<'a> {
    pub(crate) channel: &'a str,
}

/// Disables r9k mode.
pub const fn r9k_beta_off(channel: &str) -> R9kBetaOff<'_> {
    R9kBetaOff { channel }
}

impl<'a> Encodable for R9kBetaOff<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/r9kbetaoff")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn r9k_beta_off_encode() {
        test_encode(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n");
    }

    #[test]
    fn r9k_beta_off_ensure_channel_encode() {
        test_encode(r9k_beta_off("museun"), "PRIVMSG #museun :/r9kbetaoff\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn r9k_beta_off_serde() {
        test_serde(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn r9k_beta_off_ensure_channel_serde() {
        test_serde(r9k_beta_off("museun"), "PRIVMSG #museun :/r9kbetaoff\r\n");
    }
}
