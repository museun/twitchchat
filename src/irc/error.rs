/// An invalid message was either provided, or could not be parsed
#[derive(Debug)]
#[non_exhaustive]
pub enum MessageError {
    /// Invalid command
    InvalidCommand {
        /// Expedted this command
        expected: String,
        /// But got this command
        got: String,
    },

    /// Expected a nickname attached to that message
    ExpectedNick,

    /// Expected an argument at position `pos`
    ExpectedArg {
        /// 'index' of the argument (e.g. 0)
        pos: usize,
    },

    /// expected data attached to that message
    ExpectedData,

    /// Expected a specific tag
    ExpectedTag {
        /// The tag name
        name: String,
    },

    /// Cannot parse a specific tag
    CannotParseTag {
        /// The tag name
        name: String,
        /// The parse error
        error: Box<dyn std::error::Error + Send + Sync>,
    },

    /// An incomplete message was provided
    IncompleteMessage {
        /// At index `pos`
        pos: usize,
    },

    /// An empty message was provided
    EmptyMessage,

    /// A custom error message
    Custom {
        /// The inner error
        error: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand { expected, got } => {
                write!(f, "invalid command. expected '{}' got '{}'", expected, got)
            }
            Self::ExpectedNick => write!(f, "expected a nickname attached to that message"),
            Self::ExpectedArg { pos } => write!(f, "expected arg at position: {}", pos),
            Self::ExpectedData => write!(f, "expected a data segment in the message"),
            Self::ExpectedTag { name } => write!(f, "expected tag '{}'", name),
            Self::CannotParseTag { name, error } => write!(f, "cannot parse '{}': {}", name, error),
            Self::IncompleteMessage { pos } => write!(f, "incomplete message starting at: {}", pos),
            Self::EmptyMessage => write!(f, "no message could be parsed"),

            Self::Custom { error } => write!(f, "custom error: {}", error),
        }
    }
}

impl std::error::Error for MessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CannotParseTag { error, .. } => Some(&**error),
            Self::Custom { error } => Some(&**error),
            _ => None,
        }
    }
}
