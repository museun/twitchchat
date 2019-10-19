/// An error that the [`Client`](./struct.Client.html) can return
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Connection could not be established
    #[error("cannot connect: {0}")]
    Connect(#[source] std::io::Error),
    /// Could not register with the provided [UserConfig](./struct.UserConfig.html)
    #[error("cannot send initial registration: {0}")]
    Register(Box<Self>),
    /// Could not write
    #[error("cannot write: {0}")]
    Write(#[source] std::io::Error),
    /// Could not read
    #[error("cannot read: {0}")]
    Read(#[source] std::io::Error),
    /// Invalid message received from Twitch
    #[error("invalid message, from '{0}' (trimmed)")]
    InvalidMessage(String),
    /// Invalid Nick/Pass combination
    #[error("invalid registration. check the `token` and `nick`")]
    InvalidRegistration,
    /// Channel name provided was empty
    #[error("empty channel name provided")]
    EmptyChannelName,
    /// Cannot read. This probably means you need to reconnect.
    #[error("cannot read, client should quit now")]
    CannotRead,
    /// Capability is required for this functionality
    #[error("{} are required to do that",
        .0
        .iter()
        .map(|f| format!("{:?}", f))
        .collect::<Vec<_>>()
        .join(", ")
    )]
    CapabilityRequired(Vec<crate::Capability>),
    /// Not connected to the server
    #[error("not connected to server")]
    NotConnected,
}
