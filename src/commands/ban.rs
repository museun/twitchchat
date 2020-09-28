use super::{Channel, Encodable, MaybeEmpty};
use std::io::{Result, Write};

/// Permanently prevent a user from chatting.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
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
///
/// [unban]: super::unban()
pub fn ban<'a>(channel: &'a str, username: &'a str, reason: impl Into<Option<&'a str>>) -> Ban<'a> {
    Ban {
        channel,
        username,
        reason: reason.into(),
    }
}

impl<'a> Encodable for Ban<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf,
            Channel(self.channel) =>
            "/ban {}{}", self.username, MaybeEmpty(self.reason)
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
        );
    }

    #[test]
    fn ban_ensure_channel() {
        test_encode(
            ban("museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn ban_serde() {
        test_serde(
            ban("#museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn ban_ensure_channel_serde() {
        test_serde(
            ban("museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        );
    }
}
