use super::Encodable;
use std::io::{Result, Write};

/// Join a channel. This handles prepending a leading '#' for you if you omit it.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Join<'a> {
    pub(crate) channel: &'a str,
}

/// Join a channel. This handles prepending a leading '#' for you if you omit it.
pub const fn join(channel: &str) -> Join<'_> {
    Join { channel }
}

impl<'a> Encodable for Join<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        write_nl!(buf, "JOIN {}", super::Channel(self.channel))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn join_encode() {
        test_encode(join("#museun"), "JOIN #museun\r\n");
    }

    #[test]
    fn join_ensure_channel_encode() {
        test_encode(join("museun"), "JOIN #museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn join_serde() {
        test_serde(join("#museun"), "JOIN #museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn join_ensure_channel_serde() {
        test_serde(join("museun"), "JOIN #museun\r\n");
    }
}
