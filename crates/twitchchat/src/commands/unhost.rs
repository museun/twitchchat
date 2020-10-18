use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Stop hosting another channel.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unhost<'a> {
    pub(crate) channel: &'a str,
}

/// Stop hosting another channel.
pub const fn unhost(channel: &str) -> Unhost<'_> {
    Unhost { channel }
}

impl<'a> Encodable for Unhost<'a> {
fn encode(&self, buf:&mut dyn Write) -> Result<()>
    {
        write_cmd!(buf, Channel(self.channel) => "/unhost")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unhost_encode() {
        test_encode(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n");
    }

    #[test]
    fn unhost_ensure_channel_encode() {
        test_encode(unhost("museun"), "PRIVMSG #museun :/unhost\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unhost_serde() {
        test_serde(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unhost_ensure_channel_serde() {
        test_serde(unhost("museun"), "PRIVMSG #museun :/unhost\r\n")
    }
}
