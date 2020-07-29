#![cfg_attr(debug_assertions, allow(missing_docs, dead_code, unused_imports))]
// this has to be first for the macro

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
use serde::de::{Error, IntoDeserializer, Visitor};
use serde::Deserialize;
use std::ops::{Deref, Index, Range};

#[derive(serde::Serialize)]
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

impl<'a, 'de: 'a> Deserialize<'de> for Str<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Default)]
        struct V<'a>(std::marker::PhantomData<&'a ()>);

        impl<'a, 'de: 'a> Visitor<'de> for V<'a> {
            type Value = Str<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a str")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                // have to move it because this isn't borrowed
                Ok(v.to_string().into())
            }

            fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
                Ok(v.into())
            }

            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(v.into())
            }
        }

        deserializer.deserialize_str(V::default())
    }
}

impl<'a> std::fmt::Debug for Str<'a> {
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

pub struct RawVisitor<'a, T: 'a + FromIrcMessage<'a>>(std::marker::PhantomData<&'a T>);

impl<'a, T> RawVisitor<'a, T>
where
    T: 'a + FromIrcMessage<'a>,
{
    fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'a, 'de: 'a, T> Visitor<'de> for RawVisitor<'a, T>
where
    T: 'a + FromIrcMessage<'a>,
    T::Error: std::error::Error,
{
    type Value = T;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "expected to parse '{}'",
            std::any::type_name::<T>()
        )
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        use serde::de::Error as _;
        const RAW: &str = "raw";

        let mut raw = None;
        loop {
            match map.next_entry::<&str, Str<'a>>() {
                Ok(Some((k, v))) if k == RAW => {
                    if raw.replace(v).is_some() {
                        return Err(A::Error::duplicate_field(RAW));
                    }
                }
                Ok(None) => break,
                // TODO maybe log the ignored error to make debugging easier
                _ => continue,
            };
        }

        let raw = raw.ok_or_else(|| A::Error::missing_field(RAW))?;
        let irc = IrcMessage::parse(raw);

        T::from_irc(irc).map_err(|err| {
            A::Error::custom(format!(
                "cannot parse '{}' because: {}",
                std::any::type_name::<T>(),
                err
            ))
        })
    }
}
