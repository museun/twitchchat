use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct R9kBetaOff<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> R9kBetaOff<'a> {
    pub const fn new(channel: &'a str) -> Self {
        Self { channel }
    }
}

pub fn r9k_beta_off(channel: &str) -> R9kBetaOff<'_> {
    R9kBetaOff::new(channel)
}

impl<'a> Encodable for R9kBetaOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/r9kbetaoff"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn r9k_beta_off_encode() {
        test_encode(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn r9k_beta_off_serde() {
        test_serde(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n")
    }
}
