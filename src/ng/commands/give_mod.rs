use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct GiveMod<'a> {
    pub channel: &'a str,
    pub username: &'a str,
}

pub fn give_mod<'a>(channel: &'a str, username: &'a str) -> GiveMod<'a> {
    GiveMod { channel, username }
}

impl<'a> Encodable for GiveMod<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/mod", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn give_mod_encode() {
        test_encode(
            give_mod("#museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn give_mod_serde() {
        test_serde(
            give_mod("#museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        )
    }
}
