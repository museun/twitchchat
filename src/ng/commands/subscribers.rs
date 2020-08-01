use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Subscribers<'a> {
    pub channel: &'a str,
}

pub fn subscribers(channel: &str) -> Subscribers<'_> {
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
