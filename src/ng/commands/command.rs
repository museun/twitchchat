use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Command<'a> {
    pub channel: &'a str,
    pub data: &'a str,
}

pub fn command<'a>(channel: &'a str, data: &'a str) -> Command<'a> {
    Command { data, channel }
}

impl<'a> Encodable for Command<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&self.data])
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
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn command_serde() {
        test_serde(
            command("#museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        )
    }
}
