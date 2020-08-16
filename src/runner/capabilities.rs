use std::collections::HashSet;

/// Capabiltiies Twitch acknowledged.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Capabilities {
    /// You have the [membership](https://dev.twitch.tv/docs/irc/membership) capability
    pub membership: bool,
    /// You have the [commands](https://dev.twitch.tv/docs/irc/commands) capability
    pub commands: bool,
    /// You have the [tags](https://dev.twitch.tv/docs/irc/tags) capability
    pub tags: bool,
    /// A set of unknown capabilities Twitch sent to use
    pub unknown: HashSet<String>,
}
