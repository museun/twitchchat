use super::*;

pub(crate) trait Expect {
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&self) -> Result<&str, InvalidMessage>;
    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage>;
    fn expect_data(&self) -> Result<&str, InvalidMessage>;
}

impl<'a> Expect for Message<&'a str> {
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.command != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.command.to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<&str, InvalidMessage> {
        self.prefix
            .as_ref()
            .and_then(|s| s.nick())
            .cloned()
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage> {
        self.args
            .split_whitespace()
            .nth(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, InvalidMessage> {
        self.data.ok_or_else(|| InvalidMessage::ExpectedData)
    }
}
