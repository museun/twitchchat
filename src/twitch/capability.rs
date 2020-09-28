/// Capability used to enable extra functionality with the protocol
///
/// Without any of these specified, you will just able to read/write basic messages
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Capability {
    /// Membership capability
    ///
    /// Read basic IRC messages from a Twitch channel allows you to see who is in the channel
    Membership,
    /// Tags capability
    ///
    /// Provides metadata attached to each message
    Tags,
    /// Commands capability
    ///
    /// Enables many Twitch specific commands
    Commands,
}

impl Capability {
    /// Encode this capability as a string, to be sent to the server
    pub fn encode_as_str(self) -> &'static str {
        match self {
            Self::Membership => "CAP REQ :twitch.tv/membership",
            Self::Tags => "CAP REQ :twitch.tv/tags",
            Self::Commands => "CAP REQ :twitch.tv/commands",
        }
    }

    /// Attempts to 'parse' this capability from a string
    ///
    /// This will take the form of `twitch.tv/$tag` and produce a [Capability]
    #[allow(dead_code)]
    pub(crate) fn maybe_from_str(input: &str) -> Option<Self> {
        match input {
            "twitch.tv/membership" => Some(Self::Membership),
            "twitch.tv/tags" => Some(Self::Tags),
            "twitch.tv/commands" => Some(Self::Commands),
            _ => None,
        }
    }
}
