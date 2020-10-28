use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Enables subscribers-only mode (only subscribers may chat in this channel).
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Subscribers<'a> {
    pub(crate) channel: &'a str,
}

/// Enables subscribers-only mode (only subscribers may chat in this channel).
///
/// Use [subscribers_off] to disable.
///
/// [subscribers_off]: super::subscribers_off()
pub const fn subscribers(channel: &str) -> Subscribers<'_> {
    Subscribers { channel }
}

impl<'a> Encodable for Subscribers<'a> {
fn encode(&self, buf:&mut dyn Write) -> Result<()>
    {
        write_cmd!(buf, Channel(self.channel) => "/subscribers")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn subscribers_encode() {
        test_encode(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n");
    }

    #[test]
    fn subscribers_ensure_channel_encode() {
        test_encode(subscribers("museun"), "PRIVMSG #museun :/subscribers\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_serde() {
        test_serde(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_ensure_channel_serde() {
        test_serde(subscribers("museun"), "PRIVMSG #museun :/subscribers\r\n");
    }
}
