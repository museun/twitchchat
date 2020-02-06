use std::borrow::Cow;

/// Prefix is the sender of a message
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Prefix<'t> {
    /// A user sent this message
    User {
        /// Name of the user
        nick: Cow<'t, str>,
    },
    /// The server sent this message
    Server {
        /// Name of the server
        host: Cow<'t, str>,
    },
}

impl<'t> Prefix<'t> {
    pub(super) fn parse(input: &'t str) -> Option<Self> {
        let offset = match input {
            s if s.starts_with(':') => 1,
            "tmi.twitch.tv" => 0,
            _ => return None,
        };

        let input = input[offset..input.find(' ').unwrap_or_else(|| input.len())].trim();
        let prefix = match input.find('!') {
            Some(pos) => Prefix::User {
                nick: input[..pos].into(),
            },
            None => Prefix::Server { host: input.into() },
        };
        prefix.into()
    }
}

impl<'t> Prefix<'t> {
    /// The user name in this prefix
    ///
    /// This is the name of the user if a user had sent the message
    pub fn nick(&'t self) -> Option<&Cow<'t, str>> {
        match self {
            Prefix::User { nick } => Some(nick),
            _ => None,
        }
    }

    /// The host name in this prefix
    ///
    /// This is the name of the server if the server had sent the message
    pub fn host(&'t self) -> Option<&Cow<'t, str>> {
        match self {
            Prefix::Server { host } => Some(host),
            _ => None,
        }
    }
}
