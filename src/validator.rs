use crate::{InvalidMessage, IrcMessage, StrIndex, TagIndices};

pub trait Validator {
    // TODO this is a bad name
    fn parse_tags(&self) -> TagIndices;
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&self) -> Result<StrIndex, InvalidMessage>;
    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage>;
    fn expect_arg_index(&self, nth: usize) -> Result<StrIndex, InvalidMessage>;
    fn expect_data(&self) -> Result<&str, InvalidMessage>;
    fn expect_data_index(&self) -> Result<StrIndex, InvalidMessage>;
}

impl<'a> Validator for IrcMessage<'a> {
    fn parse_tags(&self) -> TagIndices {
        self.tags
            .map(|index| TagIndices::build_indices(&self.raw[index]))
            .unwrap_or_default()
    }

    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.get_command() != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.get_command().to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<StrIndex, InvalidMessage> {
        self.prefix
            .and_then(|p| p.nick_index())
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage> {
        self.nth_arg(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_arg_index(&self, nth: usize) -> Result<StrIndex, InvalidMessage> {
        self.nth_arg_index(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, InvalidMessage> {
        self.expect_data_index().map(|index| &self.raw[index])
    }

    fn expect_data_index(&self) -> Result<StrIndex, InvalidMessage> {
        self.data.ok_or_else(|| InvalidMessage::ExpectedData)
    }
}
