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

impl<'a, T> crate::Conversion<'a> for Message<T>
where
    T: crate::StringMarker + 'a,
{
    type Borrowed = Message<&'a str>;
    type Owned = Message<String>;

    fn as_borrowed(&'a self) -> Self::Borrowed {
        Message {
            raw: self.raw.borrow(),
            tags: Tags(
                self.tags
                    .0
                    .iter()
                    .map(|(k, v)| (k.borrow(), v.borrow()))
                    .collect(),
            ),
            prefix: self.prefix.as_ref().map(|s| s.as_borrowed()),
            command: self.command.borrow(),
            args: self.args.borrow(),
            data: self.data.as_ref().map(|s| s.borrow()),
        }
    }

    fn as_owned(&self) -> Self::Owned {
        Message {
            raw: fast_to_string!(self.raw),
            tags: Tags(
                self.tags
                    .clone()
                    .into_inner()
                    .into_iter()
                    .map(|(k, v)| (fast_to_string!(k), fast_to_string!(v)))
                    .collect(),
            ),
            prefix: self.prefix.as_ref().map(|s| s.as_owned()),
            command: fast_to_string!(self.command),
            args: fast_to_string!(self.args),
            data: self.data.as_ref().map(|s| fast_to_string!(s)),
        }
    }
}
