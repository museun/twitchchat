use super::super::Str;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Prefix<'a> {
    User { nick: Str<'a> },
    Server { host: Str<'a> },
}

impl<'a> Prefix<'a> {
    pub fn is_server(&self) -> bool {
        !self.is_user()
    }

    pub fn is_user(&self) -> bool {
        matches!(self, Self::User{ .. })
    }

    pub fn get_nick<'b: 'a>(&'b self) -> Option<Str<'a>> {
        match self {
            Self::User { nick } => Some(nick),
            Self::Server { .. } => None,
        }
        .map(Str::reborrow)
    }
}

// TODO this
use super::super::{AsOwned, Reborrow};

impl<'a> Reborrow<'a> for Prefix<'a> {
    fn reborrow<'b: 'a>(this: &'b Self) -> Self {
        match this {
            Prefix::User { nick } => Prefix::User {
                nick: Str::reborrow(nick),
            },
            Prefix::Server { host } => Prefix::Server {
                host: Str::reborrow(host),
            },
        }
    }
}

impl<'a> AsOwned for Prefix<'a> {
    type Owned = Prefix<'static>;
    fn as_owned(this: &Self) -> Self::Owned {
        match this {
            Prefix::User { nick } => Prefix::User {
                nick: AsOwned::as_owned(nick),
            },
            Prefix::Server { host } => Prefix::Server {
                host: AsOwned::as_owned(host),
            },
        }
    }
}
