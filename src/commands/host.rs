use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Host another channel.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Host<'a> {
    pub(crate) source: &'a str,
    pub(crate) target: &'a str,
}

/// Host another channel.
///
/// Use [unhost] to unset host mode.
///
/// [unhost]: super::unhost()
pub const fn host<'a>(source: &'a str, target: &'a str) -> Host<'a> {
    Host { source, target }
}

impl<'a> Encodable for Host<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.source) => "/host {}", Channel(self.target))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn host_encode() {
        test_encode(
            host("#museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );
    }

    #[test]
    fn host_ensure_channel_encode() {
        test_encode(
            host("#museun", "shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );

        test_encode(
            host("museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );

        test_encode(
            host("museun", "shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn host_serde() {
        test_serde(
            host("#museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn host_ensure_channel_serde() {
        test_serde(
            host("#museun", "shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );

        test_serde(
            host("museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );

        test_serde(
            host("museun", "shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        );
    }
}
