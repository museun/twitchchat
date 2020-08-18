use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Removes a timeout on a user.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Untimeout<'a> {
    pub(crate) channel: Cow<'a, str>,
    pub(crate) username: &'a str,
}

/// Removes a timeout on a user.
pub fn untimeout<'a>(channel: &'a str, username: &'a str) -> Untimeout<'a> {
    let channel = super::make_channel(channel);
    Untimeout { channel, username }
}

impl<'a> Encodable for Untimeout<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/untimeout", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn untimeout_encode() {
        test_encode(
            untimeout("#museun", "museun"),
            "PRIVMSG #museun :/untimeout museun\r\n",
        );
        test_encode(
            command("museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn untimeout_serde() {
        test_serde(
            untimeout("#museun", "museun"),
            "PRIVMSG #museun :/untimeout museun\r\n",
        );
        test_serde(
            command("museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
    }
}
