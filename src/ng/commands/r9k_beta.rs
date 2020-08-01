use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct R9kBeta<'a> {
    pub channel: &'a str,
}

pub fn r9k_beta(channel: &str) -> R9kBeta<'_> {
    R9kBeta { channel }
}

impl<'a> Encodable for R9kBeta<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/r9kbeta"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn r9k_beta_encode() {
        test_encode(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn r9k_beta_serde() {
        test_serde(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n")
    }
}
