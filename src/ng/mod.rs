#![cfg_attr(debug_assertions, allow(missing_docs, dead_code, unused_imports))]
// this has to be first for the macro
#[macro_use]
mod serde;

// mod dispatcher;
// pub use dispatcher::{DispatchError, Dispatcher};

// mod event_map;
// pub use event_map::EventMap;

// mod event_stream;
// pub use event_stream::EventStream;

// mod encoder;
// pub use encoder::{AsyncEncoder, Encodable, Encoder};

// pub mod commands;
pub mod messages;

// pub mod channel;
// pub use channel::{Receiver, Sender};

pub mod irc;
pub use irc::{IrcMessage, Prefix, PrefixIndex, TagIndices, Tags};

use messages::FromIrcMessage;
use std::{
    fmt::Debug,
    ops::{Deref, Index, Range},
};

#[derive(::serde::Serialize)]
#[serde(untagged)]
pub enum Str<'a> {
    Owned(Box<str>), // TODO make this an Arc
    Borrowed(&'a str),
}

impl<'a> Str<'a> {
    pub fn into_owned(self) -> Str<'static> {
        match self {
            Str::Owned(s) => Str::Owned(s),
            Str::Borrowed(s) => Str::Owned(s.to_string().into_boxed_str()),
        }
    }
}

impl<'a> Debug for Str<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a> PartialEq for Str<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a> PartialEq<str> for Str<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> AsRef<str> for Str<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Str::Owned(s) => &*s,
            Str::Borrowed(s) => s,
        }
    }
}

impl<'a> Deref for Str<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> Clone for Str<'a> {
    fn clone(&self) -> Self {
        match self {
            Str::Owned(s) => Self::Owned(s.to_string().into_boxed_str()),
            Str::Borrowed(s) => Self::Borrowed(s),
        }
    }
}

impl<'a> Index<&StrIndex> for Str<'a> {
    type Output = str;
    fn index(&self, index: &StrIndex) -> &Self::Output {
        &self.as_ref()[index.as_range()]
    }
}

impl<'a> Index<StrIndex> for Str<'a> {
    type Output = str;
    fn index(&self, index: StrIndex) -> &Self::Output {
        &self.as_ref()[index.as_range()]
    }
}

impl<'a> Index<&StrIndex> for str {
    type Output = str;
    fn index(&self, index: &StrIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}

impl<'a> Index<StrIndex> for str {
    type Output = str;
    fn index(&self, index: StrIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}

impl From<String> for Str<'static> {
    fn from(data: String) -> Self {
        Str::Owned(data.into_boxed_str())
    }
}

impl<'a> From<&'a str> for Str<'a> {
    fn from(data: &'a str) -> Self {
        Str::Borrowed(data)
    }
}

impl From<Box<str>> for Str<'static> {
    fn from(data: Box<str>) -> Self {
        Str::Owned(data)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct StrIndex {
    start: usize,
    end: usize,
}

impl StrIndex {
    pub const fn raw(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn new(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    pub fn offset_by(mut self, pos: usize) -> Self {
        self.start += pos;
        self.end += pos;
        self
    }

    pub fn resize(mut self, len: usize) -> Self {
        self.end = self.start + len;
        self
    }

    pub fn truncate(mut self, len: usize) -> Self {
        self.end -= len;
        self
    }

    pub fn replace(&mut self, pos: usize) -> StrIndex {
        std::mem::replace(self, Self::new(pos))
    }

    pub fn is_empty(&self) -> bool {
        // end can never be behind start
        // so if we're past start then we're not empty
        self.start == self.end
    }

    pub fn bump_tail(&mut self) {
        self.end += 1;
    }

    pub fn as_range(self) -> Range<usize> {
        self.start..self.end
    }
}
