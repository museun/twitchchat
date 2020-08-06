#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    InvalidCommand {
        expected: String,
        got: String,
    },

    ExpectedNick,

    ExpectedArg {
        pos: usize,
    },

    ExpectedData,

    ExpectedTag {
        name: String,
    },

    CannotParseTag {
        name: String,
        error: Box<dyn std::error::Error>,
    },

    IncompleteMessage {
        pos: usize,
    },
    EmptyMessage,

    Custom {
        error: Box<dyn std::error::Error>,
    },
}

impl std::fmt::Display for Error {
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CannotParseTag { error, .. } => Some(&**error),
            Self::Custom { error } => Some(&**error),
            _ => None,
        }
    }
}
