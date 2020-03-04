/// An error returned by this crate
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// A UTF-8 decoding/encoding error
    Utf8(std::str::Utf8Error),
    /// An IO error
    Io(std::io::Error),
    /// An error when trying to parse a Message
    Decode(crate::decode::ParseError),
    /// Tried to stop a non-running client
    NotRunning,
    /// Tried to start an already running client
    AlreadyRunning,
    /// An invalid channel was provided
    InvalidChannel(crate::ChannelError),
    /// The client has been disconnected
    ClientDisconnected,
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<crate::decode::ParseError> for Error {
    fn from(err: crate::decode::ParseError) -> Self {
        Self::Decode(err)
    }
}

impl From<crate::ChannelError> for Error {
    fn from(err: crate::ChannelError) -> Self {
        Self::InvalidChannel(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8(err) => write!(f, "utf8 error: {}", err),
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::Decode(err) => write!(f, "decode error: {}", err),
            Self::NotRunning => write!(f, "tried to stop a non-running client"),
            Self::AlreadyRunning => write!(f, "tried to start an already running client"),
            Self::InvalidChannel(err) => write!(f, "an invalid channel was provided: {}", err),
            Self::ClientDisconnected => write!(f, "this client has been disconnected"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Utf8(err) => Some(err),
            Self::Io(err) => Some(err),
            Self::Decode(err) => Some(err),
            Self::InvalidChannel(err) => Some(err),
            _ => None,
        }
    }
}
