use crate::{IrcError, IrcMessage, StrIndex, TagIndices};

/// This trait is provided as an easy way of defining your own custom events.
pub trait Validator {
    /// Pre-compute the tag indices
    fn parse_tags(&self) -> TagIndices;
    /// You expect a command matching 'cmd'. See the constants in `IrcMessage`
    fn expect_command(&self, cmd: &str) -> Result<(), IrcError>;
    /// You expect a user nickname to be attached. This returns the `StrIndex` of it
    fn expect_nick(&self) -> Result<StrIndex, IrcError>;
    /// You expect an argumnet at position 'nth' (0-based). This returns the `&str` of it.
    fn expect_arg(&self, nth: usize) -> Result<&str, IrcError>;
    /// You expect an argumnet at position 'nth' (0-based). This returns the `StrIndex` of it.
    fn expect_arg_index(&self, nth: usize) -> Result<StrIndex, IrcError>;
    /// You expect data to be attached to the message. This returns the `&str` of it.
    fn expect_data(&self) -> Result<&str, IrcError>;
    /// You expect data to be attached to the message. This returns the `StrIndex` of it.
    fn expect_data_index(&self) -> Result<StrIndex, IrcError>;
}

impl<'a> Validator for IrcMessage<'a> {
    fn parse_tags(&self) -> TagIndices {
        self.tags
            .map(|index| TagIndices::build_indices(&self.raw[index]))
            .unwrap_or_default()
    }

    fn expect_command(&self, cmd: &str) -> Result<(), IrcError> {
        if self.get_command() != cmd {
            return Err(IrcError::InvalidCommand {
                expected: cmd.to_string(),
                got: self.get_command().to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<StrIndex, IrcError> {
        self.prefix
            .and_then(|p| p.nick_index())
            .ok_or_else(|| IrcError::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, IrcError> {
        self.nth_arg(nth)
            .ok_or_else(|| IrcError::ExpectedArg { pos: nth })
    }

    fn expect_arg_index(&self, nth: usize) -> Result<StrIndex, IrcError> {
        self.nth_arg_index(nth)
            .ok_or_else(|| IrcError::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, IrcError> {
        self.expect_data_index().map(|index| &self.raw[index])
    }

    fn expect_data_index(&self) -> Result<StrIndex, IrcError> {
        self.data.ok_or_else(|| IrcError::ExpectedData)
    }
}
