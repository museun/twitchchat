use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Privmsg<'a> {
    pub(crate) channel: &'a str,
    pub(crate) data: &'a str,
}

impl<'a> Privmsg<'a> {
    pub const fn new(channel: &'a str, data: &'a str) -> Self {
        Self { channel, data }
    }
}

pub fn privmsg<'a>(channel: &'a str, data: &'a str) -> Privmsg<'a> {
    Privmsg::new(channel, data)
}

impl<'a> Encodable for Privmsg<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts_term(&[&"PRIVMSG ", &self.channel, &" :", &self.data])
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
}
