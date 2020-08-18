use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Removes a ban on a user.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unban<'a> {
    pub(crate) channel: Cow<'a, str>,
    pub(crate) username: &'a str,
}

/// Removes a ban on a user.
pub fn unban<'a>(channel: &'a str, username: &'a str) -> Unban<'a> {
    let channel = super::make_channel(channel);
    Unban { channel, username }
}

impl<'a> Encodable for Unban<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/unban", &self.username])
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
        );
        test_encode(
            unban("museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unban_serde() {
        test_serde(
            unban("#museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        );
        test_serde(
            unban("museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        )
    }
}
