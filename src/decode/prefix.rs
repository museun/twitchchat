/// Prefix is the sender of a message
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Prefix<T>
where
    T: crate::StringMarker,
{
    /// A user sent this message
    User {
        /// Name of the user
        nick: T,
    },
    /// The server sent this message
    Server {
        /// Name of the server
        host: T,
    },
}

impl<'a> Prefix<&'a str> {
    pub(super) fn parse(input: &'a str) -> Option<Self> {
        let offset = if input.starts_with(':') {
            1
        } else if input == "tmi.twitch.tv" {
            0
        } else {
            return None;
        };

        let input = input[offset..input.find(' ').unwrap_or_else(|| input.len())].trim();
        let prefix = match input.find('!') {
            Some(pos) => Prefix::User {
                nick: &input[..pos],
            },
            None => Prefix::Server { host: input },
        };
        prefix.into()
    }
}

impl<T> Prefix<T>
where
    T: crate::StringMarker,
{
    /// The user name in this prefix
    ///
    /// This is the name of the user if a user had sent the message
    pub fn nick(&self) -> Option<&T> {
        match self {
            Prefix::User { nick } => Some(nick),
            _ => None,
        }
    }

    /// The host name in this prefix
    ///
    /// This is the name of the server if the server had sent the message
    pub fn host(&self) -> Option<&T> {
        match self {
            Prefix::Server { host } => Some(host),
            _ => None,
        }
    }
}
