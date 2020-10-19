use super::Encodable;
use std::io::{Result, Write};

/// Whispers a message to the username.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Whisper<'a> {
    pub(crate) username: &'a str,
    pub(crate) message: &'a str,
}

/// Whispers a message to the username.
pub const fn whisper<'a>(username: &'a str, message: &'a str) -> Whisper<'a> {
    Whisper { username, message }
}

impl<'a> Encodable for Whisper<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_jtv_cmd!(buf, "/w {} {}", self.username, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn whisper_encode() {
        test_encode(
            whisper("museun", "hello world"),
            "PRIVMSG jtv :/w museun hello world\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn whisper_serde() {
        test_serde(
            whisper("museun", "hello world"),
            "PRIVMSG jtv :/w museun hello world\r\n",
        )
    }
}
