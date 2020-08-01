use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unmod<'a> {
    pub channel: &'a str,
    pub username: &'a str,
}

pub fn unmod<'a>(channel: &'a str, username: &'a str) -> Unmod<'a> {
    Unmod { channel, username }
}

impl<'a> Encodable for Unmod<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unmod", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unmod_encode() {
        test_encode(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unmod_serde() {
        test_serde(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        )
    }
}
