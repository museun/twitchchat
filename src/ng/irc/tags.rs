use super::super::{Reborrow, Str};

#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Tags<'a> {
    map: Vec<(Str<'a>, Str<'a>)>,
}

impl<'a> std::fmt::Debug for Tags<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.map.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl<'a> Tags<'a> {
    // TODO should this be public?
    pub(crate) fn parse<'b: 'a>(input: &'b Str<'a>) -> Tags<'b> {
        if !input.starts_with('@') {
            return Self::default();
        }

        let map = input[1..]
            .split_terminator(';')
            .filter_map(|s| {
                let mut iter = s.splitn(2, '=');
                Some((iter.next()?, iter.next()?))
            })
            .map(|(k, v)| (Str::from(k), Str::from(v)))
            .collect();

        Self { map }
    }

    pub fn get<K: ?Sized>(&'a self, key: &K) -> Option<Str<'a>>
    where
        K: std::borrow::Borrow<str>,
        Str<'a>: PartialEq<K>,
    {
        self.map
            .iter()
            .find_map(|(k, v)| if *k == *key { Some(v) } else { None })
            .map(Str::reborrow)
    }

    pub fn get_ref<K: ?Sized>(&self, key: &K) -> Option<&Str<'a>>
    where
        K: std::borrow::Borrow<str>,
        Str<'a>: PartialEq<K>,
    {
        self.map
            .iter()
            .find_map(|(k, v)| if *k == *key { Some(v) } else { None })
    }

    pub fn get_parsed<K: ?Sized, E>(&self, key: &K) -> Option<E>
    where
        K: std::borrow::Borrow<str>,
        Str<'a>: PartialEq<K>,
        E: std::str::FromStr,
    {
        self.get_ref(key).and_then(|s| s.as_ref().parse().ok())
    }

    pub fn get_as_bool<K: ?Sized>(&self, key: &K) -> bool
    where
        K: std::borrow::Borrow<str>,
        Str<'a>: PartialEq<K>,
    {
        match self.get_ref(key) {
            Some(d) if &*d == "1" => true,
            Some(d) if &*d == "0" => false,
            Some(d) => d.parse().ok().unwrap_or_default(),
            None => false,
        }
    }

    pub fn iter(&'a self) -> impl Iterator<Item = (Str<'a>, Str<'a>)> + 'a {
        self.map
            .iter()
            .map(|(k, v)| (Str::reborrow(k), Str::reborrow(v)))
    }

    pub fn iter_ref<'b: 'a>(&'b self) -> impl Iterator<Item = &'b (Str<'a>, Str<'a>)> + 'b {
        self.map.iter()
    }

    pub fn into_inner(self) -> Vec<(Str<'a>, Str<'a>)> {
        self.map
    }
}

impl<'a> Reborrow<'a> for Tags<'a> {
    fn reborrow<'b: 'a>(this: &'b Self) -> Self {
        Tags {
            map: this
                .map
                .iter()
                .map(|(k, v)| (Str::reborrow(k), Str::reborrow(v)))
                .collect::<Vec<_>>(),
        }
    }
}
