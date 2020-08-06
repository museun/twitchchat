use crate::{Str, TagIndices};
use std::{borrow::Borrow, str::FromStr};

#[derive(Clone, PartialEq)]
pub struct Tags<'a> {
    pub(crate) data: &'a Str<'a>,
    pub(crate) indices: &'a TagIndices,
}

impl<'a> std::fmt::Debug for Tags<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a> Tags<'a> {
    pub fn from_data_indices(data: &'a Str<'a>, indices: &'a TagIndices) -> Self {
        Self { data, indices }
    }

    pub fn raw_tags(&self) -> &'a str {
        &*self.data
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<K>(&self, key: &K) -> Option<&'a str>
    where
        K: ?Sized + Borrow<str>,
    {
        self.indices.get(key.borrow(), &*self.data)
    }

    pub fn get_parsed<K, E>(&self, key: &K) -> Option<E>
    where
        K: ?Sized + Borrow<str>,
        E: FromStr,
    {
        self.get(key)
            .map(FromStr::from_str)
            .transpose()
            .ok()
            .flatten()
    }

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

    pub fn iter(&self) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        self.indices.iter(&self.data)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_missing_leading_at() {
        let data = Str::Borrowed("foo=bar;baz=quux");
        let indices = TagIndices::build_indices(&*data);

        let tags = Tags::from_data_indices(&data, &indices);
        assert!(tags.is_empty());
    }

    #[test]
    fn invalid_input_empty_input() {
        let inputs = &["@", ""];

        for input in inputs {
            let data = Str::Borrowed(*input);
            let indices = TagIndices::build_indices(&*data);

            let tags = Tags::from_data_indices(&data, &indices);
            assert!(tags.is_empty());
        }
    }

    #[test]
    fn get_parsed() {
        let input = Str::Borrowed("@foo=42;badges=broadcaster/1,subscriber/6");
        let indices = TagIndices::build_indices(&*input);

        let tags = Tags::from_data_indices(&input, &indices);
        assert_eq!(tags.get_parsed::<_, usize>("foo").unwrap(), 42);
        assert_eq!(
            tags.get_parsed::<_, String>("foo").unwrap(),
            "42".to_string()
        );
        assert!(tags.get_parsed::<_, bool>("foo").is_none());

        #[derive(Debug)]
        struct Badges(std::collections::HashMap<String, usize>);

        impl std::str::FromStr for Badges {
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
        let input = Str::Borrowed("@foo=42;ok=true;nope=false");
        let indices = TagIndices::build_indices(&*input);

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
            let data = Str::Borrowed(*input);
            let indices = TagIndices::build_indices(&*data);
            let tags = Tags::from_data_indices(&data, &indices);

            assert_eq!(tags.get("foo").unwrap(), "bar");
            assert_eq!(tags.get("baz").unwrap(), "");
            assert!(tags.get("non-existant").is_none());
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

        let input = Str::Borrowed(input);
        let indices = TagIndices::build_indices(&*input);

        let tags = Tags::from_data_indices(&input, &indices);

        for key in expected {
            assert!(tags.get(key).is_some(), "expected: '{}'", key);
        }
    }
}
