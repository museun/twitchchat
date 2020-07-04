use super::*;
use std::borrow::Cow;

/// An IRC-like message
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message<'t> {
    /// The raw string
    pub raw: Cow<'t, str>,
    /// Any targets found in the message
    pub tags: Tags<'t>,
    /// The prefix of the message
    pub prefix: Option<Prefix<'t>>,
    /// The command of the message
    pub command: Cow<'t, str>,
    /// Arguments to the command
    pub args: Cow<'t, str>,
    /// Any data provided
    pub data: Option<Cow<'t, str>>,
}

impl<'t> Message<'t> {
    pub(super) fn parse(input: &'t str) -> Result<Self> {
        let raw = input;
        if !input.ends_with("\r\n") {
            return Err(ParseError::IncompleteMessage { pos: 0 });
        }

        let input = &input.trim_start_matches(' ');
        let input = &input[..input.len() - 2];
        if input.is_empty() {
            return Err(ParseError::EmptyMessage);
        }

        let mut parser = Parser::new(input);
        Ok(Self {
            raw: raw.into(),
            tags: parser.tags(),
            prefix: parser.prefix(),
            command: parser.command().into(),
            args: parser.args().into(),
            data: parser.data().map(Into::into),
        })
    }

    /// Get the 'nth' arg from this Message
    pub fn arg(&self, nth: usize) -> Option<&str> {
        self.args.split_whitespace().nth(nth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_spaces() {
        for i in 0..10 {
            let s = format!("{}\r\n", " ".repeat(i));
            let msg = Message::parse(&s).unwrap_err();
            assert!(matches!(msg, ParseError::EmptyMessage));
        }
    }
}
