use crate::{IntoOwned, StrIndex};

#[derive(Default, Clone, PartialEq)]
pub struct TagIndices {
    map: Box<[(StrIndex, StrIndex)]>,
}

impl std::fmt::Debug for TagIndices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.map.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl TagIndices {
    pub fn build_indices(input: &str) -> Self {
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

        // we should allow empty values
        // but not empty keys
        if !key.is_empty() {
            map.push((key, value));
        }

        Self {
            map: map.into_boxed_slice(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    pub(crate) fn iter<'t>(
        &'t self,
        data: &'t str,
    ) -> impl Iterator<Item = (&'t str, &'t str)> + 't {
        self.map.iter().map(move |(k, v)| (&data[k], &data[v]))
    }
}

impl IntoOwned<'static> for TagIndices {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}
