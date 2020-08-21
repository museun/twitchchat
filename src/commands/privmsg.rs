use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Send a normal message to a channel
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Privmsg<'a> {
    pub(crate) channel: &'a str,
    pub(crate) msg: &'a str,
}

/// Send a normal message to a channel
pub const fn privmsg<'a>(channel: &'a str, msg: &'a str) -> Privmsg<'a> {
    Privmsg { channel, msg }
}

impl<'a> Encodable for Privmsg<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_nl!(buf, "PRIVMSG {} :{}", Channel(self.channel), self.msg)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn privmsg_encode() {
        test_encode(
            privmsg("#museun", "this is a test of a line"),
            "PRIVMSG #museun :this is a test of a line\r\n",
        );

        test_encode(
            privmsg("#museun", &"foo ".repeat(500)),
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
        );
    }

    #[test]
    fn privmsg_ensure_channel_encode() {
        test_encode(
            privmsg("museun", "this is a test of a line"),
            "PRIVMSG #museun :this is a test of a line\r\n",
        );

        test_encode(
            privmsg("museun", &"foo ".repeat(500)),
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn privmsg_serde() {
        test_serde(
            privmsg("#museun", "this is a test of a line"),
            "PRIVMSG #museun :this is a test of a line\r\n",
        );

        test_serde(
            privmsg("#museun", &"foo ".repeat(500)),
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn privmsg_ensure_channel_serde() {
        test_serde(
            privmsg("museun", "this is a test of a line"),
            "PRIVMSG #museun :this is a test of a line\r\n",
        );

        test_serde(
            privmsg("museun", &"foo ".repeat(500)),
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
        );
    }
}
