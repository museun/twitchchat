use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Removes a timeout on a user.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Untimeout<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

/// Removes a timeout on a user.
pub const fn untimeout<'a>(channel: &'a str, username: &'a str) -> Untimeout<'a> {
    Untimeout { channel, username }
}

impl<'a> Encodable for Untimeout<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/untimeout {}", self.username)
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
    }

    #[test]
    fn untimeout_ensure_channel_encode() {
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
    }

    #[test]
    #[cfg(feature = "serde")]
    fn untimeout_ensure_channel_serde() {
        test_serde(
            command("museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        );
    }
}
