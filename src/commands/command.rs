use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Sends the `command` to the `channel` (e.g. `/color #FFFFFF`)
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Command<'a> {
    pub(crate) channel: Cow<'a, str>,
    pub(crate) data: &'a str,
}

/// Sends the `command` to the `channel` (e.g. `/color #FFFFFF`)
pub fn command<'a>(channel: &'a str, data: &'a str) -> Command<'a> {
    let channel = super::make_channel(channel);
    Command { channel, data }
}

impl<'a> Encodable for Command<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&self.data])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn command_encode() {
        test_encode(
            command("#museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
        test_encode(
            command("museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn command_serde() {
        test_serde(
            command("#museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
        test_serde(
            command("museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
    }
}
