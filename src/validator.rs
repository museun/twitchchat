use crate::{
    irc::{IrcMessage, MessageError, TagIndices},
    MaybeOwnedIndex,
};

/// This trait is provided as an easy way of defining your own custom events.
pub trait Validator {
    /// Pre-compute the tag indices
    fn parse_tags(&self) -> TagIndices;
    /// You expect a command matching 'cmd'. See the constants in `IrcMessage`
    fn expect_command(&self, cmd: &str) -> Result<(), MessageError>;
    /// You expect a user nickname to be attached. This returns the `MaybeOwnedIndex` of it
    fn expect_nick(&self) -> Result<MaybeOwnedIndex, MessageError>;
    /// You expect an argumnet at position 'nth' (0-based). This returns the `&str` of it.
    fn expect_arg(&self, nth: usize) -> Result<&str, MessageError>;
    /// You expect an argumnet at position 'nth' (0-based). This returns the `MaybeOwnedIndex` of it.
    fn expect_arg_index(&self, nth: usize) -> Result<MaybeOwnedIndex, MessageError>;
    /// You expect data to be attached to the message. This returns the `&str` of it.
    fn expect_data(&self) -> Result<&str, MessageError>;
    /// You expect data to be attached to the message. This returns the `MaybeOwnedIndex` of it.
    fn expect_data_index(&self) -> Result<MaybeOwnedIndex, MessageError>;
}

impl<'a> Validator for IrcMessage<'a> {
    fn parse_tags(&self) -> TagIndices {
        self.tags
            .map(|index| TagIndices::build_indices(&self.raw[index]))
            .unwrap_or_default()
    }

    fn expect_command(&self, cmd: &str) -> Result<(), MessageError> {
        if self.get_command() != cmd {
            return Err(MessageError::InvalidCommand {
                expected: cmd.to_string(),
                got: self.get_command().to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<MaybeOwnedIndex, MessageError> {
        self.prefix
            .and_then(|p| p.nick_index())
            .ok_or_else(|| MessageError::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, MessageError> {
        self.nth_arg(nth)
            .ok_or_else(|| MessageError::ExpectedArg { pos: nth })
    }

    fn expect_arg_index(&self, nth: usize) -> Result<MaybeOwnedIndex, MessageError> {
        self.nth_arg_index(nth)
            .ok_or_else(|| MessageError::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, MessageError> {
        self.expect_data_index().map(|index| &self.raw[index])
    }

    fn expect_data_index(&self) -> Result<MaybeOwnedIndex, MessageError> {
        self.data.ok_or_else(|| MessageError::ExpectedData)
    }
}
