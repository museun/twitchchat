/// Capabilities allow you to get more data from twitch
///
/// The default, `generic` is very simplistic (basically just read/write PRIVMSGs for a channel)
///
/// While enabling `membership` + `commands` + `tags` will allow you to get a much more rich set of messages
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Capability {
    /// Generic capability, the default.
    ///
    /// Simply read basic irc messages from a twitch channel
    Generic,
    /// Membership capability
    ///
    /// Similar to Generic, but allows to see who is in the channel
    Membership,
    /// Commands capability
    ///
    /// Enables many twitch specific commands
    Commands,
    /// Tags capability
    ///
    /// Provides metadata attached to each message
    Tags,
}

impl Capability {
    pub(crate) fn get_command(self) -> Option<&'static str> {
        match self {
            Capability::Generic => None,
            Capability::Membership => Some("CAP REQ :twitch.tv/membership"),
            Capability::Commands => Some("CAP REQ :twitch.tv/commands"),
            Capability::Tags => Some("CAP REQ :twitch.tv/tags"),
        }
    }
}

impl Default for Capability {
    fn default() -> Self {
        Capability::Generic
    }
}
