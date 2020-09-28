use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Grant VIP status to a user.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Vip<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

/// Grant VIP status to a user.
///
/// Use [vips] to list the VIPs of this channel.
///
/// [vips]: super::vips()
pub const fn vip<'a>(channel: &'a str, username: &'a str) -> Vip<'a> {
    Vip { channel, username }
}

impl<'a> Encodable for Vip<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/vip {}", self.username)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn vip_encode() {
        test_encode(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n");
    }

    #[test]
    fn vip_ensure_channel_encode() {
        test_encode(vip("museun", "museun"), "PRIVMSG #museun :/vip museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn vip_serde() {
        test_serde(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn vip_ensure_channel_serde() {
        test_serde(vip("museun", "museun"), "PRIVMSG #museun :/vip museun\r\n");
    }
}
