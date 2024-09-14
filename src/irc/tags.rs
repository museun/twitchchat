use crate::{irc::TagIndices, MaybeOwned};
use std::{borrow::Borrow, str::FromStr};

/// Tags are IRCv3 message tags. Twitch uses them extensively.
///
/// This type is usually obstained temporarily from `::tags()` call on a message type.
///
/// This type is intentionall very cheap and just borrows a pre-computed set of indices and a wrapped string
#[derive(Clone, PartialEq)]
pub struct Tags<'a> {
    pub(crate) data: &'a MaybeOwned<'a>,
    pub(crate) indices: &'a TagIndices,
}

impl<'a> std::fmt::Debug for Tags<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a> Tags<'a> {
    /// Build the tags view from this borrowed `MaybeOwned` and an associated `TagIndices`
    pub fn from_data_indices(data: &'a MaybeOwned<'a>, indices: &'a TagIndices) -> Self {
        Self { data, indices }
    }

    /// Gets the raw string that represents the tags
    pub fn raw_tags(&self) -> &'a str {
        self.data
    }

    /// Returns how many tags were parsed
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns whether there are any tags
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Tries to get this `key` -- with an unescaped output string
    ///
    /// Twitch requires some characters to be escaped.
    // TODO write more documentation explaining this
    pub fn get_unescaped<K>(&self, key: &K) -> Option<MaybeOwned<'a>>
    where
        K: ?Sized + Borrow<str>,
    {
        self.indices.get_unescaped(key.borrow())
    }

    /// Tries to get this `key`
    ///
    /// # NOTE: This does not unescape the tag value.
    // TODO write more documentation explaining this
    pub fn get<K>(&self, key: &K) -> Option<&'a str>
    where
        K: ?Sized + Borrow<str>,
    {
        self.indices.get(key.borrow())
    }

    /** Tries to get the tag as a parsable [std::str::FromStr] type.

    This returns None if it cannot parse, or cannot find the tag

    ```rust
    # use twitchchat::{irc::{TagIndices, Tags}, maybe_owned::MaybeOwned};
    let input: MaybeOwned<'_> = "@foo=42;color=#1E90FF".into();
    let indices = TagIndices::build_indices(&*input).unwrap();
    let tags = Tags::from_data_indices(&input, &indices);

    // 'foo' can be parsed as a usize
    let answer: usize = tags.get_parsed("foo").unwrap();
    assert_eq!(answer, 42);

    // 'foo' can be parsed a String (this shows how to use this with a 'turbofish')
    assert_eq!(
        tags.get_parsed::<_, String>("foo").unwrap(),
        "42".to_string()
    );

    // 'foo' cannot be parsed as a bool
    assert!(tags.get_parsed::<_, bool>("foo").is_none());

    // a non-std type with a FromStr impl
    # use twitchchat::twitch::color::*;
    let color: Color = tags.get_parsed("color").unwrap();
    assert_eq!(color.rgb, RGB(0x1E, 0x90, 0xFF));
    ```
    */
    pub fn get_parsed<K, E>(&self, key: &K) -> Option<E>
    where
        K: ?Sized + Borrow<str>,
        E: FromStr,
    {
        self.get_unescaped(key)
            .as_deref()
            .map(FromStr::from_str)
            .transpose()
            .ok()
            .flatten()
    }

    /** Tries to get the tag as a bool.

    If it wasn't found it'll return false

    ```rust
    # use twitchchat::irc::{TagIndices, Tags};
    # use twitchchat::maybe_owned::MaybeOwned;
    let input: MaybeOwned<'_> = "@foo=42;ok=true;nope=false;test=1;not_test=0".into();
    let indices = TagIndices::build_indices(&*input).unwrap();
    let tags = Tags::from_data_indices(&input, &indices);

    // key 'foo' is not a bool
    assert!(!tags.get_as_bool("foo"));

    // key 'ok' is a bool and is true
    assert!(tags.get_as_bool("ok"));

    // key 'nope' is a bool but its false
    assert!(!tags.get_as_bool("nope"));

    // key 'test' is 1, which is true
    assert!(tags.get_as_bool("test"));

    // key 'not_test' is 0, which is false
    assert!(!tags.get_as_bool("not_test"));

    // missing key 'foobar' is missing, which is false
    assert!(!tags.get_as_bool("this-key-is-missing"));
    ```
    */
    pub fn get_as_bool<K>(&self, key: &K) -> bool
    where
        K: ?Sized + Borrow<str>,
    {
        match self.get(key) {
            Some("1") => true,
            Some("0") | None => false,
            Some(d) => d.parse().ok().unwrap_or(false),
        }
    }

    /// Get an iterator over all of the `key, value` pairs of tags
    pub fn iter(&self) -> TagsIter<'_> {
        TagsIter {
            inner: self,
            pos: 0,
        }
    }
}

impl<'a> IntoIterator for &'a Tags<'a> {
    type Item = (&'a str, &'a str);
    type IntoIter = TagsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TagsIter {
            inner: self,
            pos: 0,
        }
    }
}

/// An iterator over the [Tags]
#[derive(Clone)]
pub struct TagsIter<'a> {
    inner: &'a Tags<'a>,
    pos: usize,
}

impl<'a> std::fmt::Debug for TagsIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagsIter").finish()
    }
}

impl<'a> Iterator for TagsIter<'a> {
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.inner.indices.len() {
            return None;
        }

        let pos = self.pos;
        self.pos += 1;

        let (k, v) = &self.inner.indices.map.get(pos)?;
        Some((&**k, &**v))
    }
}

#[cfg(feature = "serde")]
impl<'a> ::serde::Serialize for Tags<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use ::serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// The reverse of [escape_str].
///
///
/// ```rust
/// # use twitchchat::test::*;
/// let s = r"hello world; this is\na test";
/// assert_eq!(&*unescape_str(&*escape_str(s)), s);
/// ```
///
/// Use [escape_str] to reverse this.
/// ### Format
///
/// | escaped              | character   |
/// | -------------------- | ----------- |
/// | `\:`                 | `;`         |
/// | `\s`                 | ` ` (space) |
/// | `\\`                 | `\`         |
/// | `\r`                 | `CR`        |
/// | `\n`                 | `LF`        |
/// | the character itself | --          |
///
/// [ref]: https://ircv3.net/specs/extensions/message-tags.html#escaping-values
pub fn unescape_str(s: &str) -> MaybeOwned<'_> {
    if !s.chars().any(|c| c == '\\') {
        return MaybeOwned::Borrowed(s);
    }
    let mut buf = String::with_capacity(s.len());
    let mut iter = s.chars();
    while let Some(c) = iter.next() {
        match c {
            '\\' => match iter.next() {
                Some(':') => buf.push(';'),
                Some('s') => buf.push(' '),
                Some('\\') => buf.push('\\'),
                Some('r') => buf.push('\r'),
                Some('n') => buf.push('\n'),
                Some(c) => buf.push(c),
                None => break,
            },
            c => buf.push(c),
        }
    }

    MaybeOwned::Owned(buf.into())
}

/// Escapes a string according to the [IRCv3 spec][ref]
///
/// ```rust
/// # use twitchchat::test::*;
/// let s = r"hello world; this is\na test";
/// assert_eq!(&*unescape_str(&*escape_str(s)), s);
/// ```
///
/// Use [unescape_str] to reverse this.
/// ### Format
///
/// | character   | escaped              |
/// | ----------- | -------------------- |
/// | `;`         | `\:`                 |
/// | ` ` (space) | `\s`                 |
/// | `\`         | `\\`                 |
/// | `CR`        | `\r`                 |
/// | `LF`        | `\n`                 |
/// | --          | the character itself |
///
/// [ref]: https://ircv3.net/specs/extensions/message-tags.html#escaping-values
pub fn escape_str(s: &str) -> std::borrow::Cow<'_, str> {
    const NEEDS_ESCAPE: [char; 5] = [';', ' ', '\\', '\n', '\r'];
    let n = s.chars().filter(|c| NEEDS_ESCAPE.contains(c)).count();
    if n == 0 {
        return s.into();
    }

    let mut buf = String::with_capacity(s.len() + n);
    for ch in s.chars() {
        match ch {
            ';' => buf.push_str(r"\:"),
            ' ' => buf.push_str(r"\s"),
            '\\' => buf.push_str(r"\\"),
            '\n' => buf.push_str(r"\n"),
            '\r' => buf.push_str(r"\r"),
            ch => buf.push(ch),
        }
    }

    buf.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn round_trip_escape() {
        let s = r"foo;bar and\foo\rwith\n";
        assert_eq!(unescape_str(&escape_str(s)), s);
    }

    #[test]
    fn escaped_tag() {
        let s = escape_str(r"@hello;world=abc\ndef");
        let data = MaybeOwned::Borrowed(&s);
        let indices = TagIndices::build_indices(&data).unwrap();

        let tags = Tags::from_data_indices(&data, &indices);
        assert_eq!(tags.get_unescaped("hello;world").unwrap(), r"abc\ndef");
        assert_eq!(tags.get("hello;world").unwrap(), r"abc\\ndef",);
    }

    #[test]
    fn invalid_input_missing_leading_at() {
        let data = MaybeOwned::Borrowed("foo=bar;baz=quux");
        let indices = TagIndices::build_indices(&data).unwrap();

        let tags = Tags::from_data_indices(&data, &indices);
        assert!(tags.is_empty());
    }

    #[test]
    fn invalid_input_empty_input() {
        let inputs = &["@", ""];

        for input in inputs {
            let data = MaybeOwned::Borrowed(input);
            let indices = TagIndices::build_indices(&data).unwrap();

            let tags = Tags::from_data_indices(&data, &indices);
            assert!(tags.is_empty());
        }
    }

    #[test]
    fn get_parsed() {
        let input = MaybeOwned::Borrowed("@foo=42;badges=broadcaster/1,subscriber/6");
        let indices = TagIndices::build_indices(&input).unwrap();

        let tags = Tags::from_data_indices(&input, &indices);
        assert_eq!(tags.get_parsed::<_, usize>("foo").unwrap(), 42);
        assert_eq!(
            tags.get_parsed::<_, String>("foo").unwrap(),
            "42".to_string()
        );
        assert!(tags.get_parsed::<_, bool>("foo").is_none());

        #[derive(Debug)]
        struct Badges(std::collections::HashMap<String, usize>);

        impl FromStr for Badges {
            type Err = std::convert::Infallible;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let iter = s.split_terminator(',').filter_map(|s| {
                    let mut iter = s.split('/');
                    let left = iter.next()?.to_string();
                    let right = iter.next()?.parse().ok()?;
                    Some((left, right))
                });

                Ok(Self(iter.collect()))
            }
        }

        let badges = tags.get_parsed::<_, Badges>("badges").unwrap();
        assert_eq!(*badges.0.get("subscriber").unwrap(), 6);
        assert_eq!(*badges.0.get("broadcaster").unwrap(), 1);
    }

    #[test]
    fn get_bool() {
        let input = MaybeOwned::Borrowed("@foo=42;ok=true;nope=false");
        let indices = TagIndices::build_indices(&input).unwrap();

        let tags = Tags::from_data_indices(&input, &indices);
        assert!(!tags.get_as_bool("foo"));
        assert!(tags.get_as_bool("ok"));
        assert!(!tags.get_as_bool("nope"));
    }

    #[test]
    fn parse_empty_value() {
        let inputs = &[
            "@foo=bar;baz=",
            "@baz=;foo=bar",
            "@foo=bar;baz=;quux=asdf",
            "@baz=;quux=asdf;foo=bar",
        ];

        for input in inputs {
            let data = MaybeOwned::Borrowed(input);
            let indices = TagIndices::build_indices(&data).unwrap();
            let tags = Tags::from_data_indices(&data, &indices);

            assert_eq!(tags.get("foo").unwrap(), "bar");
            assert_eq!(tags.get("baz").unwrap(), "");
            assert!(tags.get("non-existant").is_none());
        }
    }

    #[test]
    fn tags_iter() {
        let inputs = &[
            "@foo=bar;baz=",
            "@baz=;foo=bar",
            "@foo=bar;baz=;quux=asdf",
            "@baz=;quux=asdf;foo=bar",
        ];

        for input in inputs {
            let data = MaybeOwned::Borrowed(input);
            let indices = TagIndices::build_indices(&data).unwrap();
            let tags = Tags::from_data_indices(&data, &indices);

            let len = tags.into_iter().count();
            assert_eq!(len, input.chars().filter(|&c| c == ';').count() + 1);
        }
    }

    #[test]
    fn parse() {
        let input = "@badges=broadcaster/1,subscriber/6;\
        color=;\
        display-name=qa_subs_partner;\
        emotes=;\
        flags=;\
        id=b1818e3c-0005-490f-ad0a-804957ddd760;\
        login=qa_subs_partner;\
        mod=0;\
        msg-id=anonsubgift;\
        msg-param-months=3;\
        msg-param-recipient-display-name=TenureCalculator;\
        msg-param-recipient-id=135054130;\
        msg-param-recipient-user-name=tenurecalculator;\
        msg-param-sub-plan-name=t111;\
        msg-param-sub-plan=1000;\
        room-id=196450059;\
        subscriber=1;\
        system-msg=An\\sanonymous\\suser\\sgifted\\sa\\sTier\\s1\\ssub\\sto\\sTenureCalculator!\\s;\
        tmi-sent-ts=1542063432068;\
        turbo=0;\
        user-type=;\
        user-id=196450059";

        let expected = &[
            "badges",
            "color",
            "display-name",
            "emotes",
            "flags",
            "id",
            "login",
            "mod",
            "msg-id",
            "msg-param-months",
            "msg-param-recipient-display-name",
            "msg-param-recipient-id",
            "msg-param-recipient-user-name",
            "msg-param-sub-plan-name",
            "msg-param-sub-plan",
            "room-id",
            "subscriber",
            "system-msg",
            "tmi-sent-ts",
            "turbo",
            "user-type",
            "user-id",
        ];

        let input = MaybeOwned::Borrowed(input);
        let indices = TagIndices::build_indices(&input).unwrap();

        let tags = Tags::from_data_indices(&input, &indices);

        for key in expected {
            assert!(tags.get(key).is_some(), "expected: '{}'", key);
        }
    }
}
