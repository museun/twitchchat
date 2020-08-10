use crate::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

/// Enables subscribers-only mode (only subscribers may chat in this channel).
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Subscribers<'a> {
    pub(crate) channel: &'a str,
}

/// Enables subscribers-only mode (only subscribers may chat in this channel).
///
/// Use [subscribers_off] to disable.
///
/// [subscribers_off]: ./struct.Encoder.html#methodruct.html#method.subscribers_off
pub const fn subscribers(channel: &str) -> Subscribers<'_> {
    Subscribers { channel }
}

impl<'a> Encodable for Subscribers<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/subscribers"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn subscribers_encode() {
        test_encode(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_serde() {
        test_serde(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n")
    }
}
