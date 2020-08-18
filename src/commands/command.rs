use super::Channel;
use crate::Encodable;
use std::io::{Result, Write};

/// Sends the `command` to the `channel` (e.g. `/color #FFFFFF`)
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Command<'a> {
    pub(crate) channel: &'a str,
    pub(crate) data: &'a str,
}

/// Sends the `command` to the `channel` (e.g. `/color #FFFFFF`)
pub fn command<'a>(channel: &'a str, data: &'a str) -> Command<'a> {
    Command { channel, data }
}

impl<'a> Encodable for Command<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(&self.channel) => &self.data)
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
    }

    #[test]
    fn command_ensure_channel_encode() {
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
    }

    #[test]
    #[cfg(feature = "serde")]
    fn command_ensure_channel_serde() {
        test_serde(
            command("museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
    }
}
