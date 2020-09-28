use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Removes a ban on a user.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unban<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

/// Removes a ban on a user.
pub const fn unban<'a>(channel: &'a str, username: &'a str) -> Unban<'a> {
    Unban { channel, username }
}

impl<'a> Encodable for Unban<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/unban {}", self.username)
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
    }

    #[test]
    fn unban_ensure_channel_encode() {
        test_encode(
            unban("museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unban_serde() {
        test_serde(
            unban("#museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unban_ensure_channel_serde() {
        test_serde(
            unban("museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        );
    }
}
