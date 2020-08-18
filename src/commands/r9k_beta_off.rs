use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Disables r9k mode.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct R9kBetaOff<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Disables r9k mode.
pub fn r9k_beta_off(channel: &str) -> R9kBetaOff<'_> {
    let channel = super::make_channel(channel);
    R9kBetaOff { channel }
}

impl<'a> Encodable for R9kBetaOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/r9kbetaoff"])
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
