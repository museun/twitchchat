use crate::twitch::Color;
use std::collections::HashSet;

/// Your identity on Twitch.
///
/// Currently this is only updated when you connect.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Identity {
    /// An anonymous identity.
    Anonymous {
        /// The capabilities you'll have
        caps: YourCapabilities,
    },

    /// A basic identity.
    Basic {
        /// Your username
        name: String,
        /// The capabilities you'll have
        caps: YourCapabilities,
    },

    /// A full identity
    ///
    /// This has more information about your user than a `Basic` identity.
    ///
    /// This is created if you've enabled atleast `Tags` and `Commands`.
    Full {
        /// Your username
        name: String,
        /// Your user-id
        user_id: i64,
        /// Your display name, if set
        display_name: Option<String>,
        /// You display color, if set
        color: Color,
        /// The capabilities you'll have
        caps: YourCapabilities,
    },
}

impl Identity {
    /// Get your username from this identity
    ///
    /// If its anonymous, it'll be `justinfan1234`
    pub fn username(&self) -> &str {
        let (nick, _) = crate::ANONYMOUS_LOGIN;
        match self {
            Self::Anonymous { .. } => nick,
            Self::Basic { name, .. } | Self::Full { name, .. } => &*name,
        }
    }
}

/// Capabilities that Twitch acknowledged.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct YourCapabilities {
    /// You have the [membership](https://dev.twitch.tv/docs/irc/membership) capability
    pub membership: bool,
    /// You have the [commands](https://dev.twitch.tv/docs/irc/commands) capability
    pub commands: bool,
    /// You have the [tags](https://dev.twitch.tv/docs/irc/tags) capability
    pub tags: bool,
    /// A set of unknown capabilities that Twitch acknowledged
    pub unknown: HashSet<String>,
}
