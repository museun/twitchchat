/// An error returned by the [Client]
///
/// [Client]: ./struct.Client.html
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Utf8(err) => write!(f, "utf8 error: {}", err),
            Error::Io(err) => write!(f, "io error: {}", err),
            Error::Decode(err) => write!(f, "decode error: {}", err),
            Error::NotRunning => write!(f, "tried to stop a non-running client"),
            Error::AlreadyRunning => write!(f, "tried to start an already running client"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Utf8(err) => Some(err),
            Error::Io(err) => Some(err),
            Error::Decode(err) => Some(err),
            _ => None,
        }
    }
}
