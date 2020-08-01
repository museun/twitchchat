use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Whisper<'a> {
    pub(crate) username: &'a str,
    pub(crate) message: &'a str,
}

impl<'a> Whisper<'a> {
    pub const fn new(username: &'a str, message: &'a str) -> Self {
        Self { username, message }
    }
}

pub fn whisper<'a>(username: &'a str, message: &'a str) -> Whisper<'a> {
    Whisper::new(username, message)
}

impl<'a> Encodable for Whisper<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/w", &self.username, &self.message])
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
