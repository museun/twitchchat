use super::super::{Str, StrIndex};
use std::{borrow::Borrow, str::FromStr};

#[derive(Clone)]
pub struct Tags<'a> {
    pub(crate) data: &'a Str<'a>,
    pub(crate) indices: &'a TagIndices,
}

impl<'a> Tags<'a> {
    pub fn raw_tags(&self) -> &'a str {
        &*self.data
    }

    pub fn len(&self) -> usize {
        self.indices.map.len()
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

impl<'a> serde::Serialize for Tags<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(self.indices.map.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct TagIndices {
    map: Vec<(StrIndex, StrIndex)>,
}

impl std::fmt::Debug for TagIndices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.map.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl TagIndices {
    // TODO should this be public?
    pub(crate) fn parse(input: &str) -> Self {
        if !input.starts_with('@') {
            return Self::default();
        }

        enum Mode {
            Head,
            Tail,
        }

        let mut map = Vec::with_capacity(input.chars().filter(|&c| c == ';').count() + 1);
        let (mut key, mut value) = (StrIndex::new(1), StrIndex::new(1));

        let mut mode = Mode::Head;

        for (i, ch) in input.char_indices().skip(1) {
            let i = i + 1;
            match ch {
                ';' => {
                    mode = Mode::Head;
                    map.push((key.replace(i), value.replace(i)));
                }
                '=' => {
                    mode = Mode::Tail;
                    value.replace(i);
                }
                _ => {
                    match mode {
                        Mode::Head => &mut key,
                        Mode::Tail => &mut value,
                    }
                    .bump_tail();
                }
            }
        }

        if !key.is_empty() && !value.is_empty() {
            map.push((key, value));
        }

        Self { map }
    }

    pub fn get<'t>(&'t self, key: &str, data: &'t str) -> Option<&'t str> {
        self.map.iter().find_map(|(k, v)| {
            if key == &data[k] {
                Some(&data[v])
            } else {
                None
            }
        })
    }

    fn iter<'t>(&'t self, data: &'t str) -> impl Iterator<Item = (&'t str, &'t str)> + 't {
        self.map.iter().map(move |(k, v)| (&data[k], &data[v]))
    }
}
