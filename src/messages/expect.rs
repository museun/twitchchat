use super::*;
use std::borrow::Cow;

pub(crate) trait Expect<'a, 'b: 'a> {
    fn expect_command(&'b self, cmd: &'a str) -> Result<(), InvalidMessage>;
    fn expect_nick(&'b self) -> Result<Cow<'a, str>, InvalidMessage>;
    fn expect_arg(&'b self, nth: usize) -> Result<Cow<'a, str>, InvalidMessage>;
    fn expect_data(&'b self) -> Result<&'a Cow<'a, str>, InvalidMessage>;
}

impl<'a, 'b: 'a> Expect<'a, 'b> for Message<'a> {
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
            .cloned()
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&'b self, nth: usize) -> Result<Cow<'a, str>, InvalidMessage> {
        self.args
            .split_whitespace()
            .nth(nth)
            .map(Into::into)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&'b self) -> Result<&'b Cow<'a, str>, InvalidMessage> {
        self.data
            .as_ref()
            .ok_or_else(|| InvalidMessage::ExpectedData)
    }
}
