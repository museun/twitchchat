use super::Encodable;
use std::io::{Result, Write};

/// Leave a channel. This handles prepending a leading '#' for you if you omit it.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Part<'a> {
    pub(crate) channel: &'a str,
}

/// Leave a channel. This handles prepending a leading '#' for you if you omit it.
pub const fn part(channel: &str) -> Part<'_> {
    Part { channel }
}

impl<'a> Encodable for Part<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        write!(buf, "PART {}\r\n", super::Channel(self.channel))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn part_encode() {
        test_encode(part("#museun"), "PART #museun\r\n");
    }

    #[test]
    fn part_ensure_channel_encode() {
        test_encode(part("museun"), "PART #museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn part_serde() {
        test_serde(part("#museun"), "PART #museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn part_ensure_channel_serde() {
        test_serde(part("museun"), "PART #museun\r\n");
    }
}
