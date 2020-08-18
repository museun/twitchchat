use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Enables followers-only mode (only users who have followed for `duration` may chat).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Followers<'a> {
    pub(crate) channel: Cow<'a, str>,
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
/// [followers_off]: ./fn.followers_off.html
pub fn followers<'a>(channel: &'a str, duration: &'a str) -> Followers<'a> {
    let channel = super::make_channel(channel);
    Followers { channel, duration }
}

impl<'a> Encodable for Followers<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/followers", &self.duration])
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

        test_serde(
            followers("museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        )
    }
}
