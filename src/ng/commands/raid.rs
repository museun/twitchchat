use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Raid<'a> {
    pub(crate) source: &'a str,
    pub(crate) target: &'a str,
}

impl<'a> Raid<'a> {
    pub const fn new(source: &'a str, target: &'a str) -> Self {
        Self { source, target }
    }
}

pub fn raid<'a>(source: &'a str, target: &'a str) -> Raid<'a> {
    Raid::new(source, target)
}

impl<'a> Encodable for Raid<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.source, &[&"/raid", &self.target])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn raid_encode() {
        test_encode(
            raid("#museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn raid_serde() {
        test_serde(
            raid("#museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        )
    }
}
