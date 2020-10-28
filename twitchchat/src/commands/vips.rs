use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Lists the VIPs of this channel.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Vips<'a> {
    pub(crate) channel: &'a str,
}

/// Lists the VIPs of this channel.
pub const fn vips(channel: &str) -> Vips<'_> {
    Vips { channel }
}

impl<'a> Encodable for Vips<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_cmd!(buf, Channel(self.channel) => "/vips")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn vips_encode() {
        test_encode(vips("#museun"), "PRIVMSG #museun :/vips\r\n");
    }

    #[test]
    fn vips_ensure_channel_encode() {
        test_encode(vips("museun"), "PRIVMSG #museun :/vips\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn vips_serde() {
        test_serde(vips("#museun"), "PRIVMSG #museun :/vips\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn vips_ensure_channel_serde() {
        test_serde(vips("museun"), "PRIVMSG #museun :/vips\r\n");
    }
}
