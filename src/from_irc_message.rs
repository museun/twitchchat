use crate::IrcMessage;
use std::convert::Infallible;

pub trait FromIrcMessage<'a>: Sized {
    type Error;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error>;
}

impl<'a> FromIrcMessage<'a> for IrcMessage<'a> {
    type Error = Infallible;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        Ok(msg)
    }
}

#[derive(Debug)]
pub enum InvalidMessage {
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
}

impl std::fmt::Display for InvalidMessage {
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
        }
    }
}

impl std::error::Error for InvalidMessage {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InvalidMessage::CannotParseTag { error, .. } => Some(&**error),
            _ => None,
        }
    }
}
