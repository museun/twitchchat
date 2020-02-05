/// An error returned when trying to use [Parse] on a [Message] to a specific [message][msg]
///
/// [Parse]: ../trait.Parse.html
/// [Message]: ../decode/struct.Message.html
/// [msg]: ./index.html
#[derive(Debug)]
#[non_exhaustive]
pub enum InvalidMessage {
    /// An invalid command was found for this message
    InvalidCommand {
        /// Expected this command
        expected: String,
        /// Got this command
        got: String,
    },
    /// Expected a nickname attached to this message
    ExpectedNick,
    /// Expected an argument at a position in this message
    ExpectedArg {
        /// Argument position
        pos: usize,
    },
    /// Expected this message to have data attached
    ExpectedData,
}

impl std::fmt::Display for InvalidMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand { expected, got } => {
                write!(f, "invalid command. got: {} expected {}", got, expected)
            }
            Self::ExpectedNick => f.write_str("expected nickname"),
            Self::ExpectedArg { pos } => write!(f, "expected arg at {}", pos),
            Self::ExpectedData => f.write_str("expected data"),
        }
    }
}

impl std::error::Error for InvalidMessage {}
