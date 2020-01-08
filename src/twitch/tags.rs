use std::borrow::Borrow;
use std::collections::HashMap;

/// Tags are IRCv3 message tags. Twitch uses them extensively.
#[derive(PartialEq)]
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

impl<T: crate::StringMarker + Borrow<str>> Tags<T> {
    /// Tries to get the tag for this `key`
    pub fn get<K: ?Sized>(&self, key: &K) -> Option<&str>
    where
        K: Borrow<str>,
    {
        self.0.get(key.borrow()).map(Borrow::borrow)
    }

    // Take ownership of the inner HashMap
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
