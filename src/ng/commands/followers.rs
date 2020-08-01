use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Followers<'a> {
    pub channel: &'a str,
    pub duration: &'a str,
}

pub fn followers<'a>(channel: &'a str, duration: &'a str) -> Followers<'a> {
    Followers { channel, duration }
}

impl<'a> Encodable for Followers<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/followers", &self.duration])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn followers_encode() {
        test_encode(
            followers("#museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn followers_serde() {
        test_serde(
            followers("#museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        )
    }
}
