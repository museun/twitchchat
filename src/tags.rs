use std::collections::HashMap;

/// Tags are IRCv3 message tags. Twitch uses them extensively
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tags(pub(crate) HashMap<String, String>);

impl Tags {
    /// Parses a `@k=v;k=v` string into the `Tags` type
    ///
    /// Use this with caution because it doesn't validate any of the parsing logic and may panic.
    /// This is only made public for convenience of constructing `Tags` outside the normal use-case for the crate
    pub fn parse(input: &str) -> Self {
        debug_assert!(input.starts_with('@'));
        let map = input[1..].split_terminator(';').filter_map(|p| {
            let pos = p.find('=')?;
            Some((p[..pos].to_string(), p[pos + 1..].to_string()))
        });
        Self(map.collect())
    }

    /// Take ownership of the inner hashmap
    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }
}
