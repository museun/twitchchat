use crate::color::Color;
use crate::{Badge, BadgeKind};
use std::borrow::Cow;

/// Converts a type to an owned version
pub trait AsOwned: private::AsOwnedSealed {
    /// The owned type
    type Owned: 'static;
    /// Get an owned version
    fn as_owned(&self) -> Self::Owned;
}

/// Helper to reborrow a Cow without moving it
pub trait Reborrow<'a>: private::ReborrowSealed
where
    Self: 'a,
{
    /// The borrowed output
    type Output: 'a;
    /// Reborrow self.
    ///
    /// This is intended for internal use.
    fn reborrow(&self) -> Self::Output;
}

impl<'a> Reborrow<'a> for &'a Cow<'a, str> {
    type Output = Cow<'a, str>;
    fn reborrow(&self) -> Self::Output {
        match self {
            Cow::Borrowed(s) => Cow::Borrowed(*s),
            Cow::Owned(s) => Cow::Borrowed(s.as_ref()),
        }
    }
}

impl<'a> Reborrow<'a> for &'a Option<Cow<'a, str>> {
    type Output = Option<Cow<'a, str>>;
    fn reborrow(&self) -> Self::Output {
        self.as_ref().map(|s| match s {
            Cow::Borrowed(s) => Cow::Borrowed(*s),
            Cow::Owned(s) => Cow::Borrowed(s.as_ref()),
        })
    }
}

impl<'a> Reborrow<'a> for Option<&'a Cow<'a, str>> {
    type Output = Option<Cow<'a, str>>;
    fn reborrow(&self) -> Self::Output {
        self.map(|s| match s {
            Cow::Borrowed(s) => Cow::Borrowed(*s),
            Cow::Owned(s) => Cow::Borrowed(s.as_ref()),
        })
    }
}

mod private {
    pub trait AsOwnedSealed {}
    impl<T> AsOwnedSealed for T where T: crate::AsOwned {}

    pub trait ReborrowSealed {}
    impl<'a, T> ReborrowSealed for T where T: crate::Reborrow<'a> {}
}

impl AsOwned for bool {
    type Owned = Self;
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl AsOwned for usize {
    type Owned = Self;
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl<'a> AsOwned for Cow<'a, str> {
    type Owned = Cow<'static, str>;
    fn as_owned(&self) -> <Self as AsOwned>::Owned {
        match self {
            Cow::Borrowed(r) => Cow::Owned(r.to_owned().into()),
            Cow::Owned(s) => Cow::Owned(s.clone()),
        }
    }
}

impl<T: AsOwned + Clone> AsOwned for Option<T> {
    type Owned = Option<T::Owned>;
    fn as_owned(&self) -> Self::Owned {
        self.clone().map(|s| s.as_owned())
    }
}

impl<T: AsOwned + Clone> AsOwned for Vec<T> {
    type Owned = Vec<T::Owned>;
    fn as_owned(&self) -> Self::Owned {
        self.iter().cloned().map(|s| s.as_owned()).collect()
    }
}

impl AsOwned for Color {
    type Owned = Self;
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl<'t> AsOwned for BadgeKind<'t> {
    type Owned = BadgeKind<'static>;
    fn as_owned(&self) -> Self::Owned {
        match self {
            BadgeKind::Admin => BadgeKind::Admin,
            BadgeKind::Bits => BadgeKind::Bits,
            BadgeKind::Broadcaster => BadgeKind::Broadcaster,
            BadgeKind::GlobalMod => BadgeKind::GlobalMod,
            BadgeKind::Moderator => BadgeKind::Moderator,
            BadgeKind::Subscriber => BadgeKind::Subscriber,
            BadgeKind::Staff => BadgeKind::Staff,
            BadgeKind::Turbo => BadgeKind::Turbo,
            BadgeKind::Premium => BadgeKind::Premium,
            BadgeKind::VIP => BadgeKind::VIP,
            BadgeKind::Partner => BadgeKind::Partner,
            BadgeKind::Unknown(s) => BadgeKind::Unknown(s.as_owned()),
        }
    }
}

impl<'t> AsOwned for Badge<'t> {
    type Owned = Badge<'static>;
    fn as_owned(&self) -> Self::Owned {
        Badge {
            kind: self.kind.as_owned(),
            data: self.data.as_owned(),
        }
    }
}

impl<'a> AsOwned for crate::Tags<'a> {
    type Owned = crate::Tags<'static>;
    fn as_owned(&self) -> Self::Owned {
        let map = self.0.iter().map(|(k, v)| (k.as_owned(), v.as_owned()));
        crate::Tags(map.collect())
    }
}

impl<'t> AsOwned for crate::decode::Prefix<'t> {
    type Owned = crate::decode::Prefix<'static>;
    fn as_owned(&self) -> Self::Owned {
        match self {
            crate::decode::Prefix::User { nick } => crate::decode::Prefix::User {
                nick: nick.as_owned(),
            },
            crate::decode::Prefix::Server { host } => crate::decode::Prefix::Server {
                host: host.as_owned(),
            },
        }
    }
}

impl<'t> AsOwned for crate::decode::Message<'t> {
    type Owned = crate::decode::Message<'static>;
    fn as_owned(&self) -> Self::Owned {
        crate::decode::Message {
            raw: self.raw.as_owned(),
            tags: self.tags.as_owned(),
            prefix: self.prefix.as_owned(),
            command: self.command.as_owned(),
            args: self.args.as_owned(),
            data: self.data.as_owned(),
        }
    }
}
