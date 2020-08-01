use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Slow<'a> {
    pub(crate) channel: &'a str,
    pub(crate) duration: usize,
}

impl<'a> Slow<'a> {
    pub fn new(channel: &'a str, duration: impl Into<Option<usize>>) -> Self {
        Self {
            channel,
            duration: duration.into().unwrap_or(120),
        }
    }
}

pub fn slow(channel: &str, duration: impl Into<Option<usize>>) -> Slow<'_> {
    Slow::new(channel, duration)
}

impl<'a> Encodable for Slow<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/slow", &self.duration.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn slow_encode() {
        test_encode(slow("#museun", Some(42)), "PRIVMSG #museun :/slow 42\r\n");
        test_encode(slow("#museun", 42), "PRIVMSG #museun :/slow 42\r\n");
        test_encode(slow("#museun", None), "PRIVMSG #museun :/slow 120\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn slow_serde() {
        test_serde(slow("#museun", Some(42)), "PRIVMSG #museun :/slow 42\r\n");
        test_serde(slow("#museun", 42), "PRIVMSG #museun :/slow 42\r\n");
        test_serde(slow("#museun", None), "PRIVMSG #museun :/slow 120\r\n");
    }
}
