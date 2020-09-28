use super::{Channel, Encodable, MaybeEmpty};
use std::io::{Result, Write};

/// Temporarily prevent a user from chatting.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Timeout<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
    pub(crate) duration: Option<&'a str>,
    pub(crate) reason: Option<&'a str>,
}

/// Temporarily prevent a user from chatting.
///
/// * duration (*optional*, default=`10 minutes`) must be a positive integer.
/// * time unit (*optional*, default=`s`) must be one of
///   * s
///   * m
///   * h
///   * d
///   * w
/// * maximum duration is `2 weeks`.
///
/// Combinations like `1d2h` are also allowed.
///
/// Reason is optional and will be shown to the target user and other moderators.
///
/// Use [untimeout] to remove a timeout.
///
/// [untimeout]: super::untimeout()
pub fn timeout<'a>(
    channel: &'a str,
    username: &'a str,
    duration: impl Into<Option<&'a str>>,
    reason: impl Into<Option<&'a str>>,
) -> Timeout<'a> {
    Timeout {
        channel,
        username,
        duration: duration.into(),
        reason: reason.into(),
    }
}

impl<'a> Encodable for Timeout<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel)=>
            "/timeout {}{}{}",
            self.username,
            MaybeEmpty(self.duration),
            MaybeEmpty(self.reason),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn timeout_encode() {
        test_encode(
            timeout("#museun", "museun", None, None),
            "PRIVMSG #museun :/timeout museun\r\n",
        );
        test_encode(
            timeout("#museun", "museun", Some("1d2h"), None),
            "PRIVMSG #museun :/timeout museun 1d2h\r\n",
        );
        test_encode(
            timeout("#museun", "museun", None, Some("spamming")),
            "PRIVMSG #museun :/timeout museun spamming\r\n",
        );
        test_encode(
            timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
            "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
        );
    }

    #[test]
    fn timeout_ensure_channel_encode() {
        test_encode(
            timeout("museun", "museun", None, None),
            "PRIVMSG #museun :/timeout museun\r\n",
        );
        test_encode(
            timeout("museun", "museun", Some("1d2h"), None),
            "PRIVMSG #museun :/timeout museun 1d2h\r\n",
        );
        test_encode(
            timeout("museun", "museun", None, Some("spamming")),
            "PRIVMSG #museun :/timeout museun spamming\r\n",
        );
        test_encode(
            timeout("museun", "museun", Some("1d2h"), Some("spamming")),
            "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn timeout_serde() {
        test_serde(
            timeout("#museun", "museun", None, None),
            "PRIVMSG #museun :/timeout museun\r\n",
        );
        test_serde(
            timeout("#museun", "museun", Some("1d2h"), None),
            "PRIVMSG #museun :/timeout museun 1d2h\r\n",
        );
        test_serde(
            timeout("#museun", "museun", None, Some("spamming")),
            "PRIVMSG #museun :/timeout museun spamming\r\n",
        );
        test_serde(
            timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
            "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn timeout_ensure_channel_serde() {
        test_serde(
            timeout("museun", "museun", None, None),
            "PRIVMSG #museun :/timeout museun\r\n",
        );
        test_serde(
            timeout("museun", "museun", Some("1d2h"), None),
            "PRIVMSG #museun :/timeout museun 1d2h\r\n",
        );
        test_serde(
            timeout("museun", "museun", None, Some("spamming")),
            "PRIVMSG #museun :/timeout museun spamming\r\n",
        );
        test_serde(
            timeout("museun", "museun", Some("1d2h"), Some("spamming")),
            "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
        );
    }
}
