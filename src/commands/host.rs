use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Host another channel.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Host<'a> {
    pub(crate) source: Cow<'a, str>,
    pub(crate) target: Cow<'a, str>,
}

/// Host another channel.
///
/// Use [unhost] to unset host mode.
///
/// [unhost]: ./struct.Encoder.html#method.unhost
pub fn host<'a>(source: &'a str, target: &'a str) -> Host<'a> {
    let source = super::make_channel(source);
    let target = super::make_channel(target);
    Host { source, target }
}

impl<'a> Encodable for Host<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.source, &[&"/host", &&*self.target])
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
        test_encode(
            host("museun", "shaken_bot"),
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
        test_serde(
            host("museun", "shaken_bot"),
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
