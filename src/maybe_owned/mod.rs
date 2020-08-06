use std::{fmt::Debug, ops::Deref};

#[macro_use]
mod into_owned;
pub use into_owned::IntoOwned;

mod maybe_owned_index;
pub use maybe_owned_index::MaybeOwnedIndex;

#[cfg_attr(feature = "serde", derive(::serde::Serialize), serde(untagged))]
pub enum MaybeOwned<'a> {
    Owned(Box<str>),
    Borrowed(&'a str),
}

impl<'a> MaybeOwned<'a> {
    pub fn is_owned(&self) -> bool {
        !self.is_borrowed()
    }

    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed{..})
    }
}

impl<'a> Clone for MaybeOwned<'a> {
    fn clone(&self) -> MaybeOwned<'a> {
        match self {
            Self::Owned(s) => Self::Owned(s.to_string().into_boxed_str()),
            Self::Borrowed(s) => Self::Borrowed(s),
        }
    }
}

impl<'a> Debug for MaybeOwned<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a> PartialEq for MaybeOwned<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a> PartialEq<str> for MaybeOwned<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&str> for MaybeOwned<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> AsRef<str> for MaybeOwned<'a> {
    fn as_ref(&self) -> &str {
        match self {
            MaybeOwned::Owned(s) => &*s,
            MaybeOwned::Borrowed(s) => s,
        }
    }
}

impl<'a> Deref for MaybeOwned<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<String> for MaybeOwned<'static> {
    fn from(data: String) -> Self {
        MaybeOwned::Owned(data.into_boxed_str())
    }
}

impl<'a> From<&'a str> for MaybeOwned<'a> {
    fn from(data: &'a str) -> Self {
        MaybeOwned::Borrowed(data)
    }
}

impl From<Box<str>> for MaybeOwned<'static> {
    fn from(data: Box<str>) -> Self {
        MaybeOwned::Owned(data)
    }
}
