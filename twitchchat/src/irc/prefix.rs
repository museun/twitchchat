use crate::{IntoOwned, Str, StrIndex};

/// Prefix is the sender of a message
pub struct Prefix<'a> {
    pub(crate) data: &'a Str<'a>,
    pub(crate) index: PrefixIndex,
}

impl<'a> std::fmt::Debug for Prefix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data[self.index.as_index()].fmt(f)
    }
}

impl<'a> Prefix<'a> {
    /// Was this message from the server?
    pub fn is_server(&self) -> bool {
        !self.is_user()
    }

    /// Was this message from a user?
    pub fn is_user(&self) -> bool {
        matches!(self.index, PrefixIndex::User{ .. })
    }

    /// Get the full, raw string
    pub fn get_raw_prefix(&self) -> &'a str {
        &self.data[self.index.as_index()]
    }

    // Get the nickname of this prefix, if it was sent by a user
    pub fn get_nick(&self) -> Option<&'a str> {
        self.index.nick_index().map(|index| &self.data[index])
    }
}

/// Prefix is the sender of a message
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum PrefixIndex {
    /// A user prefix
    User {
        /// Index of the nickname
        nick: StrIndex,
    },
    /// A server prefix
    Server {
        /// Index of the hostname
        host: StrIndex,
    },
}

impl PrefixIndex {
    /// Was this message from the server?
    pub fn is_server(&self) -> bool {
        !self.is_nick()
    }

    /// Was this message from a user?
    pub fn is_nick(&self) -> bool {
        matches!(self, Self::User{ .. })
    }

    /// Get the index of the nickname
    pub fn nick_index(self) -> Option<StrIndex> {
        match self {
            Self::User { nick } => Some(nick),
            Self::Server { .. } => None,
        }
    }

    /// Get the index of the hostname
    pub fn host_index(self) -> Option<StrIndex> {
        match self {
            Self::Server { host } => Some(host),
            Self::User { .. } => None,
        }
    }

    /// Consumes this returning the index
    pub fn as_index(self) -> StrIndex {
        match self {
            Self::User { nick } => nick,
            Self::Server { host } => host,
        }
    }
}

impl IntoOwned<'static> for PrefixIndex {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}
