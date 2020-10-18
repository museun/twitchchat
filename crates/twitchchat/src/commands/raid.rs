use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Raid another channel.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Raid<'a> {
    pub(crate) source: &'a str,
    pub(crate) target: &'a str,
}

/// Raid another channel.
///
/// Use [unraid] to cancel the Raid.
///
/// [unraid]: super::unraid()
pub const fn raid<'a>(source: &'a str, target: &'a str) -> Raid<'a> {
    Raid { source, target }
}

impl<'a> Encodable for Raid<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.source) => "/raid {}", Channel(self.target))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn raid_encode() {
        test_encode(
            raid("#museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
    }

    #[test]
    fn raid_ensure_channel_encode() {
        test_encode(
            raid("museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );

        test_encode(
            raid("#museun", "museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
        test_encode(
            raid("museun", "museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn raid_serde() {
        test_serde(
            raid("#museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn raid_ensure_channel_serde() {
        test_serde(
            raid("museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
        test_serde(
            raid("#museun", "museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
        test_serde(
            raid("museun", "museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        );
    }
}
