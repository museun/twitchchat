use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unvip<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

impl<'a> Unvip<'a> {
    pub const fn new(channel: &'a str, username: &'a str) -> Self {
        Self { channel, username }
    }
}

pub fn unvip<'a>(channel: &'a str, username: &'a str) -> Unvip<'a> {
    Unvip::new(channel, username)
}

impl<'a> Encodable for Unvip<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unvip", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unvip_encode() {
        test_encode(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unvip_serde() {
        test_serde(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        )
    }
}
