use super::Channel;
use crate::Encodable;
use std::io::{Result, Write};

/// Reply to a specific message (using an UUID) on a channel
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Reply<'a> {
    pub(crate) channel: &'a str,
    pub(crate) msg_id: &'a str,
    pub(crate) msg: &'a str,
}

/// Reply to a specific message (using an UUID) on a channel
pub const fn reply<'a>(channel: &'a str, msg_id: &'a str, msg: &'a str) -> Reply<'a> {
    Reply {
        channel,
        msg_id,
        msg,
    }
}

impl<'a> Encodable for Reply<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        write_nl!(
            buf,
            "@reply-parent-msg-id={} PRIVMSG {} :{}",
            self.msg_id,
            Channel(self.channel),
            self.msg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    const TEST_UUID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    #[test]
    fn reply_encode() {
        test_encode(
            reply("#museun", TEST_UUID, "this is a test of a line"),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :this is a test of a line\r\n",
                TEST_UUID
            ),
        );

        test_encode(
            reply("#museun", TEST_UUID, &"foo ".repeat(500)),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :{}\r\n",
                TEST_UUID,
                &"foo ".repeat(500)
            ),
        );
    }

    #[test]
    fn reply_ensure_channel_encode() {
        test_encode(
            reply("museun", TEST_UUID, "this is a test of a line"),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :this is a test of a line\r\n",
                TEST_UUID
            ),
        );

        test_encode(
            reply("museun", TEST_UUID, &"foo ".repeat(500)),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :{}\r\n",
                TEST_UUID,
                &"foo ".repeat(500)
            ),
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn reply_serde() {
        test_serde(
            reply("#museun", TEST_UUID, "this is a test of a line"),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :this is a test of a line\r\n",
                TEST_UUID
            ),
        );

        test_serde(
            reply("#museun", TEST_UUID, &"foo ".repeat(500)),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :{}\r\n",
                TEST_UUID,
                &"foo ".repeat(500)
            ),
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn reply_ensure_channel_serde() {
        test_serde(
            reply("museun", TEST_UUID, "this is a test of a line"),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :this is a test of a line\r\n",
                TEST_UUID
            ),
        );

        test_serde(
            reply("museun", TEST_UUID, &"foo ".repeat(500)),
            format!(
                "@reply-parent-msg-id={} PRIVMSG #museun :{}\r\n",
                TEST_UUID,
                &"foo ".repeat(500)
            ),
        );
    }
}
