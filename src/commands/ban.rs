use crate::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

/// Permanently prevent a user from chatting.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Ban<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
    pub(crate) reason: Option<&'a str>,
}

/// Permanently prevent a user from chatting.
///
/// Reason is optional and will be shown to the target user and other moderators.
///
/// Use [unban] to remove a ban.
pub fn ban<'a>(channel: &'a str, username: &'a str, reason: impl Into<Option<&'a str>>) -> Ban<'a> {
    Ban {
        channel,
        username,
        reason: reason.into(),
    }
}

impl<'a> Encodable for Ban<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            self.channel,
            &[&"/ban", &self.username, &self.reason.unwrap_or_default()],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn ban_encode() {
        test_encode(
            ban("#museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn ban_serde() {
        test_serde(
            ban("#museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        )
    }
}
