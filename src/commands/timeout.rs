use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Temporarily prevent a user from chatting.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Timeout<'a> {
    pub(crate) channel: Cow<'a, str>,
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
/// [untimeout]: ./fn.untimeout.html
pub fn timeout<'a>(
    channel: &'a str,
    username: &'a str,
    duration: impl Into<Option<&'a str>>,
    reason: impl Into<Option<&'a str>>,
) -> Timeout<'a> {
    let channel = super::make_channel(channel);
    Timeout {
        channel,
        username,
        duration: duration.into(),
        reason: reason.into(),
    }
}

impl<'a> Encodable for Timeout<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            &&*self.channel,
            &[
                &"/timeout",
                &self.username,
                &self.duration.unwrap_or_default(),
                &self.reason.unwrap_or_default(),
            ],
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
