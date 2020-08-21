use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Revoke VIP status from a user.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unvip<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

/// Revoke VIP status from a user.
///
/// Use [vips] to list the VIPs of this channel.
///
/// [vips]: ./fn.vips.html
pub const fn unvip<'a>(channel: &'a str, username: &'a str) -> Unvip<'a> {
    Unvip { channel, username }
}

impl<'a> Encodable for Unvip<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/unvip {}", self.username)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unvip_encode() {
        test_encode(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }

    #[test]
    fn unvip_ensure_channel_encode() {
        test_encode(
            unvip("museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unvip_serde() {
        test_serde(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unvip_ensure_channel_serde() {
        test_serde(
            unvip("museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }
}
