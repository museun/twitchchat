use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Host<'a> {
    pub(crate) source: &'a str,
    pub(crate) target: &'a str,
}

impl<'a> Host<'a> {
    pub const fn new(source: &'a str, target: &'a str) -> Self {
        Self { source, target }
    }
}

pub fn host<'a>(source: &'a str, target: &'a str) -> Host<'a> {
    Host::new(source, target)
}

impl<'a> Encodable for Host<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.source, &[&"/host", &self.target])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn host_encode() {
        test_encode(
            host("#museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn host_serde() {
        test_serde(
            host("#museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        )
    }
}
