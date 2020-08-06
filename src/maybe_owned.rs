use crate::color::Color;
use std::{
    fmt::Debug,
    ops::{Deref, Index, Range},
};

macro_rules! into_owned {
    ($ty:ident { $($field:ident),* $(,)? }) => {
        impl<'a> crate::IntoOwned<'a> for $ty<'a> {
            type Output = $ty<'static>;
            fn into_owned(self) -> Self::Output {
                $ty {
                    $(
                      $field: self.$field.into_owned(),
                    )*
                }
            }
        }
    };
}

/// Converts a 'borrowed' type into an owned type. e.g. 'a to 'static
pub trait IntoOwned<'a> {
    /// The output type
    type Output: 'static;
    /// Consumes self, returning an owned version
    fn into_owned(self) -> Self::Output;
}

impl<'a> IntoOwned<'a> for MaybeOwned<'a> {
    type Output = MaybeOwned<'static>;
    fn into_owned(self) -> Self::Output {
        match self {
            Self::Owned(s) => MaybeOwned::Owned(s),
            Self::Borrowed(s) => MaybeOwned::Owned(s.to_string().into_boxed_str()),
        }
    }
}

impl IntoOwned<'static> for MaybeOwnedIndex {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

impl IntoOwned<'static> for Color {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

impl<'a, T: IntoOwned<'a> + 'a> IntoOwned<'a> for Option<T> {
    type Output = Option<T::Output>;
    fn into_owned(self) -> Self::Output {
        self.map(IntoOwned::into_owned)
    }
}

macro_rules! into_owned_primitives {
    ($($ty:ty)*) => {
        $(
            impl IntoOwned<'static> for $ty {
                type Output = Self;
                fn into_owned(self) -> Self::Output {
                    self
                }
            }
        )*
    };
}

into_owned_primitives! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    bool f32 f64
}

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

type IndexWidth = u16;

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MaybeOwnedIndex {
    pub start: IndexWidth,
    pub end: IndexWidth,
}

// TODO document this
impl MaybeOwnedIndex {
    pub const fn raw(start: usize, end: usize) -> Self {
        Self {
            start: start as IndexWidth,
            end: end as IndexWidth,
        }
    }

    pub const fn new(pos: usize) -> Self {
        Self {
            start: pos as IndexWidth,
            end: pos as IndexWidth,
        }
    }

    pub const fn offset_by(mut self, pos: usize) -> Self {
        self.start += pos as IndexWidth;
        self.end += pos as IndexWidth;
        self
    }

    pub const fn resize(mut self, len: usize) -> Self {
        self.end = self.start + len as IndexWidth;
        self
    }

    pub const fn truncate(mut self, len: usize) -> Self {
        self.end -= len as IndexWidth;
        self
    }

    pub fn replace(&mut self, pos: usize) -> MaybeOwnedIndex {
        std::mem::replace(self, Self::new(pos))
    }

    pub const fn is_empty(&self) -> bool {
        // end can never be behind start
        // so if we're past start then we're not empty
        self.start == self.end
    }

    pub fn bump_tail(&mut self) {
        self.end += 1;
    }

    pub const fn as_range(self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }
}

impl<'a> Index<&MaybeOwnedIndex> for MaybeOwned<'a> {
    type Output = str;
    fn index(&self, index: &MaybeOwnedIndex) -> &Self::Output {
        &self.as_ref()[index.as_range()]
    }
}

impl<'a> Index<MaybeOwnedIndex> for MaybeOwned<'a> {
    type Output = str;
    fn index(&self, index: MaybeOwnedIndex) -> &Self::Output {
        &self.as_ref()[index.as_range()]
    }
}

impl<'a> Index<&MaybeOwnedIndex> for str {
    type Output = str;
    fn index(&self, index: &MaybeOwnedIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}

impl<'a> Index<MaybeOwnedIndex> for str {
    type Output = str;
    fn index(&self, index: MaybeOwnedIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}
