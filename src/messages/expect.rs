use super::error::InvalidMessage;
use crate::decode::Message;
use crate::Reborrow;

use std::borrow::Cow;

pub trait Expect<'b: 'a, 'a> {
    fn expect_command(&'b self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&'b self) -> Result<Cow<'a, str>, InvalidMessage>;
    fn expect_arg(&'b self, nth: usize) -> Result<Cow<'a, str>, InvalidMessage>;
    fn expect_data(&'b self) -> Result<Cow<'a, str>, InvalidMessage>;
    fn expect_data_ref(&'b self) -> Result<&'b Cow<'a, str>, InvalidMessage>;
}

impl<'b: 'a, 'a> Expect<'b, 'a> for Message<'a> {
    fn expect_command(&'b self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.command != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.command.to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&'b self) -> Result<Cow<'a, str>, InvalidMessage> {
        self.prefix
            .as_ref()
            .and_then(|s| s.nick())
            .reborrow()
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&'b self, nth: usize) -> Result<Cow<'a, str>, InvalidMessage> {
        self.args
            .split_whitespace()
            .nth(nth)
            .map(Into::into)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&'b self) -> Result<Cow<'a, str>, InvalidMessage> {
        self.data
            .as_ref()
            .map(|s| s.reborrow())
            .ok_or_else(|| InvalidMessage::ExpectedData)
    }

    fn expect_data_ref(&'b self) -> Result<&'b Cow<'a, str>, InvalidMessage> {
        self.data
            .as_ref()
            .ok_or_else(|| InvalidMessage::ExpectedData)
    }
}
