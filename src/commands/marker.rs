use super::{Channel, Encodable, MaybeEmpty};
use std::io::{Result, Write};

/// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Marker<'a> {
    pub(crate) channel: &'a str,
    pub(crate) comment: Option<&'a str>,
}

/// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
///
/// You can use markers in the Highlighter for easier editing.
///
/// If the string exceeds 140 characters then it will be truncated
pub fn marker<'a>(channel: &'a str, comment: impl Into<Option<&'a str>>) -> Marker<'_> {
    Marker {
        channel,
        comment: comment.into(),
    }
}

impl<'a> Encodable for Marker<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        fn truncate(s: &str) -> &str {
            const MAX: usize = 140;
            if s.len() <= MAX {
                return s;
            }

            for n in (0..=MAX).rev() {
                if s.is_char_boundary(n) {
                    return &s[..n];
                }
            }

            ""
        }

        write_cmd!(buf, Channel(self.channel) => "/marker{}", MaybeEmpty(self.comment.map(truncate)))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn marker_encode() {
        test_encode(
            marker("#museun", Some("this is an example")),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_encode(
            marker("#museun", "this is an example"),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_encode(
            marker("#museun", "a".repeat(200).as_str()),
            format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
        );
        test_encode(marker("#museun", None), "PRIVMSG #museun :/marker\r\n");
    }

    #[test]
    fn marker_ensure_channel_encode() {
        test_encode(
            marker("museun", Some("this is an example")),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_encode(
            marker("museun", "this is an example"),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_encode(
            marker("museun", "a".repeat(200).as_str()),
            format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
        );
        test_encode(marker("museun", None), "PRIVMSG #museun :/marker\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn marker_serde() {
        test_serde(
            marker("#museun", Some("this is an example")),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_serde(
            marker("#museun", "this is an example"),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_serde(
            marker("#museun", "a".repeat(200).as_str()),
            format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
        );
        test_serde(marker("#museun", None), "PRIVMSG #museun :/marker\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn marker_ensure_channel_serde() {
        test_serde(
            marker("museun", Some("this is an example")),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_serde(
            marker("museun", "this is an example"),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_serde(
            marker("museun", "a".repeat(200).as_str()),
            format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
        );
        test_serde(marker("museun", None), "PRIVMSG #museun :/marker\r\n");
    }
}
