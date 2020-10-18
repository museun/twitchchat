use super::{Channel, Encodable, MaybeEmpty};
use std::io::{Result, Write};

/// Triggers a commercial.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Commercial<'a> {
    pub(crate) channel: &'a str,
    pub(crate) length: Option<usize>,
}

/// Triggers a commercial.
///
/// Length *(optional)* must be a positive number of seconds.
pub fn commercial(channel: &str, length: impl Into<Option<usize>>) -> Commercial<'_> {
    Commercial {
        channel,
        length: length.into(),
    }
}

impl<'a> Encodable for Commercial<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        let length = self.length.map(|s| s.to_string());
        write_cmd!(buf, Channel(self.channel) => "/commercial{}", MaybeEmpty(length.as_deref()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn commercial_encode() {
        test_encode(
            commercial("#museun", None),
            "PRIVMSG #museun :/commercial\r\n",
        );
        test_encode(
            commercial("#museun", 10),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
        test_encode(
            commercial("#museun", Some(10)),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
    }

    #[test]
    fn commercial_ensure_channel_encode() {
        test_encode(
            commercial("museun", None),
            "PRIVMSG #museun :/commercial\r\n",
        );
        test_encode(
            commercial("museun", 10),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
        test_encode(
            commercial("museun", Some(10)),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn commercial_serde() {
        test_serde(
            commercial("#museun", None),
            "PRIVMSG #museun :/commercial\r\n",
        );
        test_serde(
            commercial("#museun", 10),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
        test_serde(
            commercial("#museun", Some(10)),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn commercial_ensure_channel_serde() {
        test_serde(
            commercial("museun", None),
            "PRIVMSG #museun :/commercial\r\n",
        );
        test_serde(
            commercial("museun", 10),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
        test_serde(
            commercial("museun", Some(10)),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
    }
}
