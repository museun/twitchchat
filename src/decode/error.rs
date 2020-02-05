/// An error occured while parsing a line.
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ParseError {
    /// An empty line was found
    EmptyLine,

    /// An incomplete message was parsed
    IncompleteMessage {
        /// Position of the start of this invalid message
        pos: usize,
    },

    /// An empty message was parsed
    EmptyMessage,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLine => f.write_str("an empty line was attempted to be decoded"),
            Self::IncompleteMessage { pos } => {
                write!(f, "an incomplete message was parsed. at {}", pos)
            }
            Self::EmptyMessage => f.write_str("an empty message was parsed"),
        }
    }
}

impl std::error::Error for ParseError {}
