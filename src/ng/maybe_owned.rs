#![cfg_attr(debug_assertions, allow(missing_docs))]

pub trait Reborrow<'a>
where
    Self: 'a,
{
    fn reborrow<'b: 'a>(this: &'b Self) -> Self;
}

#[derive(Clone, Ord, Eq, Hash)]
pub enum MaybeOwned<'a> {
    Borrowed(&'a str),
    Owned(Box<str>),
}

impl<'a> MaybeOwned<'a> {
    pub fn reborrow<'b: 'a>(this: &'b MaybeOwned<'a>) -> MaybeOwned<'b> {
        match this {
            Self::Borrowed(s) => Self::Borrowed(s),
            Self::Owned(t) => Self::Borrowed(&*t),
        }
    }

    pub fn is_owned(&self) -> bool {
        !self.is_borrowed()
    }

    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed { .. })
    }

    pub fn into_owned(self) -> Box<str> {
        match self {
            Self::Borrowed(s) => s.into(),
            Self::Owned(s) => s,
        }
    }
}

impl<'a> PartialEq<String> for MaybeOwned<'a> {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other
    }
}

impl<'a, 'b> PartialEq<&'b str> for MaybeOwned<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<str> for MaybeOwned<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq for MaybeOwned<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a> PartialOrd for MaybeOwned<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a> std::fmt::Debug for MaybeOwned<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(r#""{}""#, self.as_ref().escape_debug()))
    }
}

impl<'a> AsRef<str> for MaybeOwned<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(t) => &*t,
        }
    }
}

impl<'a> std::borrow::Borrow<str> for MaybeOwned<'a> {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl<'a> std::ops::Deref for MaybeOwned<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Borrowed(s) => s,
            MaybeOwned::Owned(t) => &*t,
        }
    }
}

impl<'a> std::fmt::Display for MaybeOwned<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a> From<MaybeOwned<'a>> for std::borrow::Cow<'a, str> {
    fn from(s: MaybeOwned<'a>) -> Self {
        match s {
            MaybeOwned::Borrowed(s) => Self::Borrowed(s),
            MaybeOwned::Owned(t) => Self::Owned(t.into()),
        }
    }
}

impl<'a> From<&'a str> for MaybeOwned<'a> {
    fn from(input: &'a str) -> Self {
        Self::Borrowed(input)
    }
}

impl<'a> From<&'a String> for MaybeOwned<'a> {
    fn from(input: &'a String) -> Self {
        Self::Borrowed(input.as_str())
    }
}

impl From<String> for MaybeOwned<'static> {
    fn from(input: String) -> Self {
        Self::Owned(input.into())
    }
}

impl From<Box<str>> for MaybeOwned<'static> {
    fn from(input: Box<str>) -> Self {
        Self::Owned(input)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for MaybeOwned<'a> {
    fn serialize<S>(&self, serialize: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize.serialize_str(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de: 'a, 'a> serde::Deserialize<'de> for MaybeOwned<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};
        struct MaybeOwnedVisitor;

        impl<'d> Visitor<'d> for MaybeOwnedVisitor {
            type Value = MaybeOwned<'d>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a borrowed string")
            }

            fn visit_borrowed_str<E: Error>(self, v: &'d str) -> Result<Self::Value, E> {
                Ok(MaybeOwned::Borrowed(v))
            }

            fn visit_borrowed_bytes<E: Error>(self, v: &'d [u8]) -> Result<Self::Value, E> {
                std::str::from_utf8(v)
                    .map_err(|_| Error::invalid_value(Unexpected::Bytes(v), &self))
                    .map(Self::Value::from)
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                // TODO it should be using transient references
                Ok(MaybeOwned::Owned(v.to_string().into()))
            }

            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                // TODO it should be using transient references
                Ok(MaybeOwned::Owned(v.into()))
            }
        }

        deserializer.deserialize_str(MaybeOwnedVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Str;
    use super::*;

    #[test]
    fn reborrow() {
        let asdf = String::from("asdf").into();

        struct Foo<'a> {
            inner: Str<'a>,
        }

        impl<'a> Reborrow<'a> for Foo<'a> {
            fn reborrow<'b: 'a>(this: &'b Self) -> Self {
                Foo {
                    inner: Str::reborrow(&this.inner),
                }
            }
        }

        fn try_it<'b: 'a, 'a>(p: &'b Foo<'b>) -> Foo<'a> {
            Reborrow::reborrow(p)
        }

        fn try_it_cov<'b: 'a, 'a>(p: &'b Foo<'a>) -> Foo<'a> {
            Reborrow::reborrow(p)
        }

        let left = Foo { inner: asdf };

        let right = try_it(&left);
        assert!(std::ptr::eq(left.inner.as_ptr(), right.inner.as_ptr()));

        let right = try_it_cov(&left);
        assert!(std::ptr::eq(left.inner.as_ptr(), right.inner.as_ptr()));
    }
}
