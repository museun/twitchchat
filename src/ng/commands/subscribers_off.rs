use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SubscribersOff<'a> {
    pub channel: &'a str,
}

pub fn subscribers_off(channel: &str) -> SubscribersOff<'_> {
    SubscribersOff { channel }
}

impl<'a> Encodable for SubscribersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/subscribersoff"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn subscribers_off_encode() {
        test_encode(
            subscribers_off("#museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn subscribers_off_serde() {
        test_serde(
            subscribers_off("#museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        )
    }
}
