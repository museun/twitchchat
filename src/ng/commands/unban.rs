use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unban<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

impl<'a> Unban<'a> {
    pub const fn new(channel: &'a str, username: &'a str) -> Self {
        Self { channel, username }
    }
}

pub fn unban<'a>(channel: &'a str, username: &'a str) -> Unban<'a> {
    Unban::new(channel, username)
}

impl<'a> Encodable for Unban<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unban", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unban_encode() {
        test_encode(
            unban("#museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unban_serde() {
        test_serde(
            unban("#museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        )
    }
}
