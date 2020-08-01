use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct FollowersOff<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> FollowersOff<'a> {
    pub const fn new(channel: &'a str) -> Self {
        Self { channel }
    }
}

pub fn followers_off(channel: &str) -> FollowersOff<'_> {
    FollowersOff::new(channel)
}

impl<'a> Encodable for FollowersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/followersoff"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn followers_off_encode() {
        test_encode(
            followers_off("#museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn followers_off_serde() {
        test_serde(
            followers_off("#museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        )
    }
}
