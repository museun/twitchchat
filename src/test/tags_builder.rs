use std::borrow::Cow;
use std::collections::HashMap;

use crate::irc::{TagIndices, Tags};
use crate::MaybeOwned;

#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_copy_implementations)]
/// An error returned by the Tags builder
pub enum BuilderError {
    /// An empty key was provided
    EmptyKey,
    /// An empty set of tags was provided
    EmptyTags,
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyKey => f.write_str("an empty key was provided"),
            Self::EmptyTags => f.write_str("an empty set of tags was provided"),
        }
    }
}

impl std::error::Error for BuilderError {}

/// A builder for Tags -- this is useful for testing
///
/// ```rust
/// # use twitchchat::irc::{Tags, TagIndices};
/// # use twitchchat::twitch::{color::RGB, Color};
/// use twitchchat::test::TagsBuilder;
///
/// // create a builder
/// let user_tags = TagsBuilder::new()
///      // and add some key-values
///     .add("color", "#F0F0F0")
///     .add("display-name", "some-fancy-name")
///      // it'll escape both keys and values
///     .add("my-message", "my\nmessage\nspans\nmultiple\nlines")
///      // and return a type you can keep around
///     .build()
///      // or an error if you provided empty keys / or no keys
///     .unwrap();
///
/// // get the 'normal' tags from this type
/// let tags = user_tags.as_tags();
///
/// let color = tags.get_parsed::<_, Color>("color").unwrap();
/// assert_eq!(color.rgb, RGB(0xF0, 0xF0, 0xF0));
///
/// assert_eq!(tags.get("display-name").unwrap(), "some-fancy-name");
///
/// // if the value was escaped, get will returned the escaped string
/// assert_eq!(tags.get("my-message").unwrap(), r"my\nmessage\nspans\nmultiple\nlines");
///
/// // you can get the unescaped value with `get_unescaped`
/// assert_eq!(tags.get_unescaped("my-message").unwrap(), "my\nmessage\nspans\nmultiple\nlines");
/// ```
#[derive(Default, Debug, Clone)]
pub struct TagsBuilder<'a> {
    // the spec says 'last' key wins, and the order is irrelevant.
    // so lets just use a hashmap
    tags: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> TagsBuilder<'a> {
    /// Create a new TagsBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add this `key` with this `value` to the builder
    ///
    /// # NOTE
    /// `value` can be empty.
    /// `key` will replace any previous keys
    ///
    pub fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Merge these pre-parsed tags into this collection
    ///
    /// # NOTE
    /// This'll override any previously set keys.
    pub fn merge(mut self, tags: &Tags<'_>) -> Self {
        self.tags.extend(
            tags.iter()
                .map(|(k, v)| (Cow::Owned(k.to_owned()), Cow::Owned(v.to_owned()))),
        );
        self
    }

    /// Build the tags reference string and its indices.
    ///
    /// # Errors
    /// If any empty keys were found, or no keys at all then an error will be returned.
    pub fn build(self) -> Result<UserTags, BuilderError> {
        use std::fmt::Write as _;
        if self.tags.is_empty() {
            return Err(BuilderError::EmptyTags);
        }

        let mut buf = String::from("@");

        for (i, (k, v)) in self.tags.iter().enumerate() {
            if k.is_empty() {
                return Err(BuilderError::EmptyKey);
            }

            if i > 0 {
                buf.push(';')
            }

            write!(
                &mut buf,
                "{key}={val}",
                key = super::escape_str(k),
                val = super::escape_str(v)
            )
            .expect("memory for string allocation");
        }

        let indices = TagIndices::build_indices(&buf).map_err(|err| match err {
            crate::MessageError::MissingTagKey(_) => BuilderError::EmptyKey,
            crate::MessageError::MissingTagValue(_) => BuilderError::EmptyTags,
            _ => unreachable!(),
        })?;

        Ok(UserTags {
            data: buf.into(),
            indices,
        })
    }
}

/// Tags built by the user
#[derive(Clone, Debug, PartialEq)]
pub struct UserTags {
    /// The rendered string
    ///
    /// This is in the form of '@key=val;key=val' without the trailing space, but with the leading '@'
    pub data: MaybeOwned<'static>,
    /// Indices of tags in the string
    pub indices: TagIndices,
}

impl UserTags {
    /// Get these tags as the 'normal' tags type
    pub fn as_tags(&self) -> Tags<'_> {
        Tags::from_data_indices(&self.data, &self.indices)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use {super::*, crate::irc::tags};

    #[test]
    fn escape() {
        let tests = &[
            // chars
            (";", r"\:"),
            (" ", r"\s"),
            (r"\", r"\\"),
            (r"\n", r"\\n"),
            (r"\r", r"\\r"),
            // strings
            (r"\\", r"\\\\"),
            ("hello;", r"hello\:"),
            (" hello world", r"\shello\sworld"),
            (r"\something", r"\\something"),
            (r"the_end\n", r"the_end\\n"),
            (r"the_win_end\r", r"the_win_end\\r"),
        ];
        for (input, expected) in tests {
            assert_eq!(tags::escape_str(input), *expected)
        }

        let tests = &["dont_escape+me", "foo=1234"];
        for test in tests {
            assert_eq!(tags::escape_str(test), *test)
        }
    }

    #[test]
    fn tags_builder() {
        let mut builder = TagsBuilder::new();

        let tests = &[("hello", "world"), ("and", "another thing"), ("len", "42")];
        for (k, v) in tests {
            builder = builder.add(*k, *v);
        }

        let user_tags = builder.build().unwrap();
        let tags = user_tags.as_tags();

        for (k, v) in tests {
            assert_eq!(tags.get_unescaped(k).unwrap(), *v)
        }

        assert_eq!(tags.get_parsed::<_, i32>("len").unwrap(), 42);

        assert!(matches!(
            TagsBuilder::new().build().unwrap_err(),
            BuilderError::EmptyTags
        ));

        let user_tags = TagsBuilder::new().add("empty", "").build().unwrap();
        let tags = user_tags.as_tags();
        assert_eq!(tags.get_unescaped("empty").unwrap(), "");

        for escaped in &[" ", r"\n", r"\r", r"\", r"hello;"] {
            let user_tags = TagsBuilder::new().add(*escaped, "").build().unwrap();
            let tags = user_tags.as_tags();
            assert_eq!(tags.get_unescaped(*escaped).unwrap(), "");
        }

        assert!(matches!(
            TagsBuilder::new().add("", "hello").build().unwrap_err(),
            BuilderError::EmptyKey
        ));
    }

    #[test]
    fn merge() {
        use crate::FromIrcMessage as _;

        let msg = "@badge-info=;badges=broadcaster/1;color=#FF69B4;display-name=museun;emote-only=1;emotes=25:0-4,6-10/81274:12-17;flags=;id=4e160a53-5482-4764-ba28-f224cd59a51f;mod=0;room-id=23196011;subscriber=0;tmi-sent-ts=1601079032426;turbo=0;user-id=23196011;user-type= :museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :Kappa Kappa VoHiYo\r\n";
        let msg = crate::IrcMessage::parse(MaybeOwned::Borrowed(msg)).unwrap();
        let pm = crate::messages::Privmsg::from_irc(msg).unwrap();
        let tags = pm.tags();

        {
            let user_tags = TagsBuilder::new().merge(&tags).build().unwrap();
            let new_tags = user_tags.as_tags();

            // this ensures they are sorted the same for the tests
            let old = tags.iter().collect::<BTreeMap<_, _>>();
            let new = new_tags.iter().collect::<BTreeMap<_, _>>();
            assert_eq!(old, new)
        }

        // merging overrides previously set tags
        {
            let user_tags = TagsBuilder::new()
                .add("color", "#FF00FF")
                .merge(&tags)
                .build()
                .unwrap();
            let tags = user_tags.as_tags();
            assert_eq!(tags.get_unescaped("color").unwrap(), "#FF69B4");
        }

        // adding overrides previously set tags
        {
            let user_tags = TagsBuilder::new()
                .merge(&tags)
                .add("color", "#FF00FF")
                .build()
                .unwrap();
            let tags = user_tags.as_tags();
            assert_eq!(tags.get_unescaped("color").unwrap(), "#FF00FF");
        }

        {
            let user_tags = TagsBuilder::new()
                .add("color", "#FF00FF")
                .add("color", "#FF0000")
                .build()
                .unwrap();
            let tags = user_tags.as_tags();
            assert_eq!(tags.get_unescaped("color").unwrap(), "#FF0000");
        }
    }
}
