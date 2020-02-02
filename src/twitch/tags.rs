use std::borrow::Borrow;
use std::collections::HashMap;

/// Tags are IRCv3 message tags. Twitch uses them extensively.
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tags<T: crate::StringMarker>(pub(crate) HashMap<T, T>);

impl<T: crate::StringMarker> Default for Tags<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: crate::StringMarker> std::fmt::Debug for Tags<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

impl<T: crate::StringMarker> Clone for Tags<T> {
    fn clone(&self) -> Self {
        Tags(self.0.clone())
    }
}

impl<'a> Tags<&'a str> {
    /**
    Parses a `@k=v;k=v` string into a `Tags` type

    # WARNING
    Use this with caution because it doesn't valid any of the parsing logic and may panic.

    This is only made public for convenience of construction `Tags` outside the normal use-case for this crate.

    This isn't fully [IRCv3] compliant

    [IRCv3]: https://ircv3.net/specs/extensions/message-tags.html
    */
    pub fn parse(input: &'a str) -> Option<Self> {
        if !input.starts_with('@') || input.len() < 2 {
            return None;
        }
        let map = input[1..].split_terminator(';').filter_map(|part| {
            let pos = part.find('=')?;
            (&part[..pos], &part[pos + 1..]).into()
        });
        Self(map.collect()).into()
    }
}

impl<T: crate::StringMarker> Tags<T> {
    /// Tries to get the tag for this `key`
    pub fn get<K: ?Sized>(&self, key: &K) -> Option<&T>
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
    pub fn iter(&self) -> impl Iterator<Item = (&T, &T)> + '_ {
        self.0.iter()
    }

    /// Get an iterator over the keys in the tags
    pub fn keys(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.keys()
    }

    /// Get an iterator over the values in the tags
    pub fn values(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.values()
    }
}

impl<T> Tags<T>
where
    T: crate::StringMarker,
{
    /// Take ownership of the inner HashMap
    pub fn into_inner(self) -> HashMap<T, T> {
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

        let tags = Tags::parse(input).unwrap();
        tags.get("badges").unwrap();
        tags.get("color").unwrap();
        tags.get("display-name").unwrap();
        tags.get("emotes").unwrap();
        tags.get("flags").unwrap();
        tags.get("id").unwrap();
        tags.get("login").unwrap();
        tags.get("mod").unwrap();
        tags.get("msg-id").unwrap();
        tags.get("msg-param-months").unwrap();
        tags.get("msg-param-recipient-display-name").unwrap();
        tags.get("msg-param-recipient-id").unwrap();
        tags.get("msg-param-recipient-user-name").unwrap();
        tags.get("msg-param-sub-plan-name").unwrap();
        tags.get("msg-param-sub-plan").unwrap();
        tags.get("room-id").unwrap();
        tags.get("subscriber").unwrap();
        tags.get("system-msg").unwrap();
        tags.get("tmi-sent-ts").unwrap();
        tags.get("turbo").unwrap();
        tags.get("user-id").unwrap();
        tags.get("user-type").unwrap();
    }
}
