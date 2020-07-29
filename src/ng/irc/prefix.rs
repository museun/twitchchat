use super::super::{Str, StrIndex};

// TODO is this borrow going to be a problem?
pub struct Prefix<'a> {
    pub(crate) data: &'a Str<'a>,
    pub(crate) index: PrefixIndex,
}

impl<'a> std::fmt::Debug for Prefix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.data[self.index.as_index()])
    }
}

impl<'a> Prefix<'a> {
    pub fn is_server(&self) -> bool {
        !self.is_user()
    }

    pub fn is_user(&self) -> bool {
        matches!(self.index, PrefixIndex::User{ .. })
    }

    pub fn get_prefix(&self) -> &'a str {
        &self.data[self.index.as_index()]
    }

    pub fn get_nick(&self) -> Option<&'a str> {
        self.index.nick_index().map(|index| &self.data[index])
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum PrefixIndex {
    User { nick: StrIndex },
    Server { host: StrIndex },
}

impl PrefixIndex {
    pub fn nick_index(self) -> Option<StrIndex> {
        match self {
            PrefixIndex::User { nick } => Some(nick),
            PrefixIndex::Server { .. } => None,
        }
    }

    pub fn as_index(self) -> StrIndex {
        match self {
            PrefixIndex::User { nick } => nick,
            PrefixIndex::Server { host } => host,
        }
    }
}
