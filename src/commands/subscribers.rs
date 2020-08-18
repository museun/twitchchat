use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Enables subscribers-only mode (only subscribers may chat in this channel).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Subscribers<'a> {
    pub(crate) channel: Cow<'a, str>,
}

/// Enables subscribers-only mode (only subscribers may chat in this channel).
///
/// Use [subscribers_off] to disable.
///
/// [subscribers_off]: ./fn.subscribers_off.html
pub fn subscribers(channel: &str) -> Subscribers<'_> {
    let channel = super::make_channel(channel);
    Subscribers { channel }
}

impl<'a> Encodable for Subscribers<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/subscribers"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn subscribers_encode() {
        test_encode(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n");
        test_encode(subscribers("museun"), "PRIVMSG #museun :/subscribers\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_serde() {
        test_serde(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n");
        test_serde(subscribers("museun"), "PRIVMSG #museun :/subscribers\r\n");
    }
}
