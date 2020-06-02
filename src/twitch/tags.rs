use crate::Reborrow;

use std::borrow::{Borrow, Cow};
use std::collections::HashMap;

/// Tags are IRCv3 message tags. Twitch uses them extensively.
#[derive(PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tags<'t>(pub(crate) HashMap<Cow<'t, str>, Cow<'t, str>>);

impl<'t> std::fmt::Debug for Tags<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

impl<'t> Tags<'t> {
    /**
    Parses a `@k=v;k=v` string into a `Tags` type

    # WARNING
    Use this with caution because it doesn't valid any of the parsing logic and may panic.

    This is only made public for convenience of construction `Tags` outside the normal use-case for this crate.

    This isn't fully [IRCv3] compliant

    [IRCv3]: https://ircv3.net/specs/extensions/message-tags.html
    */
    pub fn parse(input: &'t str) -> Option<Self> {
        if !input.starts_with('@') || input.len() < 2 {
            return None;
        }
        let map = input[1..].split_terminator(';').filter_map(|part| {
            let pos = part.find('=')?;
            (part[..pos].into(), part[pos + 1..].into()).into()
        });
        Self(map.collect()).into()
    }

    /// Tries to get the tag for this `key`.
    ///
    /// # Note
    /// This doesn't `clone`, but rather reborrows the key as another `Cow`
    pub fn get<K: ?Sized>(&'t self, key: &K) -> Option<Cow<'t, str>>
    where
        K: Borrow<str>,
    {
        self.0.get(key.borrow()).reborrow()
    }

    /// Tries to get a reference to the tag for this `key`
    ///
    /// # Note
    /// This is provided so you don't have to play as much type-tetris
    pub fn get_ref<K: ?Sized>(&'t self, key: &K) -> Option<&'t Cow<'t, str>>
    where
        K: Borrow<str>,
    {
        self.0.get(key.borrow())
    }

    /** Tries to get the tag as a parsable [`FromStr`] type.

    This returns None if it cannot parse, or cannot find the tag

    [FromStr]: https://doc.rust-lang.org/std/str/trait.FromStr.html

    ```rust
    # use twitchchat::Tags;
    let input = "@foo=42;color=#1E90FF";
    let tags = Tags::parse(input).unwrap();

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
    # use twitchchat::color::*;
    let color: Color = tags.get_parsed("color").unwrap();
    assert_eq!(color.rgb, RGB(0x1E, 0x90, 0xFF));
    ```
    */
    pub fn get_parsed<K: ?Sized, E>(&self, key: &K) -> Option<E>
    where
        K: Borrow<str>,
        E: std::str::FromStr,
    {
        self.get(key).and_then(|s| s.as_ref().parse().ok())
    }

    /** Tries to get the tag as a bool.

    If it wasn't found it'll return false

    ```rust
    # use twitchchat::Tags;
    let input = "@foo=42;ok=true;nope=false";
    let tags = Tags::parse(input).unwrap();

    // 'foo' is not a bool
    assert!(!tags.get_as_bool("foo"));

    // 'ok' is a bool and is true
    assert!(tags.get_as_bool("ok"));

    // 'nope' is a bool but its false
    assert!(!tags.get_as_bool("nope"));
    ```
    */
    pub fn get_as_bool<K: ?Sized>(&self, key: &K) -> bool
    where
        K: Borrow<str>,
    {
        self.get_parsed(key).unwrap_or_default()
    }

    /// Get an iterator over the key,value pairs in the tags
    ///
    /// # Note
    /// This doesn't `clone`, but rather reborrows the key as another `Cow`
    pub fn iter(&'t self) -> impl Iterator<Item = (Cow<'t, str>, Cow<'t, str>)> + '_ {
        self.0.iter().map(|(k, v)| (k.reborrow(), v.reborrow()))
    }

    /// Get an iterator over the key,value pairs in the tags
    ///
    /// # Note
    /// This is provided so you don't have to play as much type-tetris
    pub fn iter_ref(&self) -> impl Iterator<Item = (&Cow<'t, str>, &Cow<'t, str>)> + '_ {
        self.0.iter()
    }

    /// Get an iterator over the keys in the tags
    ///
    /// # Note
    /// This doesn't `clone`, but rather reborrows the key as another `Cow`
    pub fn keys(&'t self) -> impl Iterator<Item = Cow<'t, str>> + '_ {
        self.0.keys().map(|s| s.reborrow())
    }

    /// Get an iterator over the keys in the tags
    ///
    /// # Note
    /// This is provided so you don't have to play as much type-tetris
    pub fn keys_ref(&self) -> impl Iterator<Item = &Cow<'t, str>> + '_ {
        self.0.keys()
    }

    /// Get an iterator over the values in the tags
    ///
    /// # Note
    /// This doesn't `clone`, but rather reborrows the key as another `Cow`
    pub fn values(&'t self) -> impl Iterator<Item = Cow<'t, str>> + '_ {
        self.0.values().map(|s| s.reborrow())
    }

    /// Get an iterator over the values in the tags
    ///
    /// # Note
    /// This is provided so you don't have to play as much type-tetris
    pub fn values_ref(&self) -> impl Iterator<Item = &Cow<'t, str>> + '_ {
        self.0.values()
    }

    /// Take ownership of the inner HashMap
    pub fn into_inner(self) -> HashMap<Cow<'t, str>, Cow<'t, str>> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_missing_leading_at() {
        assert!(Tags::parse("foo=bar;baz=quux").is_none());
    }

    #[test]
    fn invalid_input_empty_input() {
        assert!(Tags::parse("@").is_none());
        assert!(Tags::parse("").is_none());
    }

    #[test]
    fn get_parsed() {
        let input = "@foo=42;badges=broadcaster/1,subscriber/6";
        let tags = Tags::parse(input).unwrap();
        assert_eq!(tags.get_parsed::<_, usize>("foo").unwrap(), 42);
        assert_eq!(
            tags.get_parsed::<_, String>("foo").unwrap(),
            "42".to_string()
        );
        assert!(tags.get_parsed::<_, bool>("foo").is_none());

        use std::collections::HashMap;
        #[derive(Debug)]
        struct Badges(HashMap<String, usize>);
        impl std::str::FromStr for Badges {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self {
                    0: s.split_terminator(',')
                        .filter_map(|s| {
                            let mut iter = s.split('/');
                            let left = iter.next()?.to_string();
                            let right = iter.next()?.parse::<usize>().ok()?;
                            (left, right).into()
                        })
                        .collect(),
                })
            }
        }

        let badges = tags.get_parsed::<_, Badges>("badges").unwrap();
        assert_eq!(*badges.0.get("subscriber").unwrap(), 6);
        assert_eq!(*badges.0.get("broadcaster").unwrap(), 1);
    }

    #[test]
    fn get_bool() {
        let input = "@foo=42;ok=true;nope=false";
        let tags = Tags::parse(input).unwrap();
        assert!(!tags.get_as_bool("foo"));
        assert!(tags.get_as_bool("ok"));
        assert!(!tags.get_as_bool("nope"));
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
        user-id=196450059;\
        user-type=";

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
            "user-id",
            "user-type",
        ];
        let tags = Tags::parse(input).unwrap();
        for expected in expected {
            tags.get(expected).unwrap();
        }
    }
}
