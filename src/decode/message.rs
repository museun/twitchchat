use super::*;

/// An IRC-like message
#[derive(Debug, PartialEq, Clone)]
pub struct Message<T>
where
    T: crate::StringMarker,
{
    /// The raw string
    pub raw: T,
    /// Any targets found in the message
    pub tags: Tags<T>,
    /// The prefix of the message
    pub prefix: Option<Prefix<T>>,
    /// The command of the message
    pub command: T,
    /// Arguments to the command
    pub args: T,
    /// Any data provided
    pub data: Option<T>,
}

impl<'a> Message<&'a str> {
    pub(super) fn parse(input: &'a str) -> Result<Self> {
        let raw = input;
        if !input.ends_with("\r\n") {
            return Err(ParseError::IncompleteMessage { pos: 0 });
        }

        let input = &input.trim_start_matches(' ')[..input.len() - 2];
        if input.is_empty() {
            return Err(ParseError::EmptyMessage);
        }

        let mut parser = Parser::new(input);
        Ok(Self {
            raw,
            tags: parser.tags(),
            prefix: parser.prefix(),
            command: parser.command(),
            args: parser.args(),
            data: parser.data(),
        })
    }

    pub fn arg(&self, nth: usize) -> Option<&str> {
        self.args.split_whitespace().nth(nth)
    }
}
