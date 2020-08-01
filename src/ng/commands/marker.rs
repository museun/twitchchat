use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Marker<'a> {
    pub(crate) channel: &'a str,
    pub(crate) comment: Option<&'a str>,
}

impl<'a> Marker<'a> {
    pub fn new(channel: &'a str, comment: impl Into<Option<&'a str>>) -> Self {
        Self {
            channel,
            comment: comment.into(),
        }
    }
}

pub fn marker<'a>(channel: &'a str, comment: impl Into<Option<&'a str>>) -> Marker<'_> {
    Marker::new(channel, comment)
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

        ByteWriter::new(buf).command(
            self.channel,
            &[&"/marker", &self.comment.map(truncate).unwrap_or_default()],
        )
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
}
