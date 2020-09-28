use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Enables followers-only mode (only users who have followed for `duration` may chat).
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Followers<'a> {
    pub(crate) channel: &'a str,
    pub(crate) duration: &'a str,
}

/// Enables followers-only mode (only users who have followed for `duration` may chat).
///
/// Examples: `"30m"`, `"1 week"`, `"5 days 12 hours"`.
///
/// Must be less than 3 months.
///
/// Use [followers_off] to disable.
///
/// [followers_off]: super::followers_off()
pub const fn followers<'a>(channel: &'a str, duration: &'a str) -> Followers<'a> {
    Followers { channel, duration }
}

impl<'a> Encodable for Followers<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/followers {}", self.duration)
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
        );
    }

    #[test]
    fn followers_ensure_channel_encode() {
        test_encode(
            followers("museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn followers_serde() {
        test_serde(
            followers("#museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn followers_ensure_channel_serde() {
        test_serde(
            followers("museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        );
    }
}
