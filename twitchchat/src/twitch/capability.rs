#[derive(Debug)]
#[non_exhaustive]
/// An error returned by [`std::str::FromStr`] for [`Capability`]
pub struct CapabilityParseError {
    cap: String,
}

impl std::fmt::Display for CapabilityParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown capability: {}", self.cap.escape_debug())
    }
}

impl std::error::Error for CapabilityParseError {}

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
}

impl std::str::FromStr for Capability {
    type Err = CapabilityParseError;

    /// Currently only these caps are supported:
    ///
    /// * "twitch.tv/membership"
    /// * "twitch.tv/tags"
    /// * "twitch.tv/commands"
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let this = match input {
            "twitch.tv/membership" => Self::Membership,
            "twitch.tv/tags" => Self::Tags,
            "twitch.tv/commands" => Self::Commands,
            cap => {
                let cap = cap.to_string();
                return Err(CapabilityParseError { cap });
            }
        };
        Ok(this)
    }
}
