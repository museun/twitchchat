use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Vip<'a> {
    pub channel: &'a str,
    pub username: &'a str,
}

pub fn vip<'a>(channel: &'a str, username: &'a str) -> Vip<'a> {
    Vip { channel, username }
}

impl<'a> Encodable for Vip<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/vip", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn vip_encode() {
        test_encode(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn vip_serde() {
        test_serde(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n")
    }
}
