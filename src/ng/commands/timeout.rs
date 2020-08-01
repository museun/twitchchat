use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Timeout<'a> {
    pub channel: &'a str,
    pub username: &'a str,
    pub duration: Option<&'a str>,
    pub reason: Option<&'a str>,
}

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
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            self.channel,
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
}
