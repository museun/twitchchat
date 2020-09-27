use crate::irc::tag_indices::helper::TagBuilder;
use crate::{IntoOwned, MaybeOwnedIndex};

/// Pre-computed tag indices
///
/// This type is only exposed for those wanting to extend/make custom types.
#[derive(Default, Clone, PartialEq)]
pub struct TagIndices {
    pub(super) map: Box<[(MaybeOwnedIndex, MaybeOwnedIndex)]>,
}

impl std::fmt::Debug for TagIndices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.map.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl TagIndices {
    /// Build indices from this tags fragment
    ///
    /// The fragment should be in the form of `'@k1=v2;k2=v2'`    
    pub fn build_indices(input: &str) -> Self {
        if !input.starts_with('@') {
            return Self::default();
        }

        enum Mode {
            Head,
            Tail,
        }

        let mut map = Vec::with_capacity(input.chars().filter(|&c| c == ';').count() + 1);
        let (mut key, mut value) = (MaybeOwnedIndex::new(1), MaybeOwnedIndex::new(1));

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

    /// Using TagBuilder
    /// TODO: add multiple builders TagBuilder?
    pub fn with_builder() {
        let _builder = TagBuilder::new();
        unimplemented!();
    }

    /// Get the number of parsed tags
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Checks whether any tags were parsed
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // NOTE: this isn't public because they don't verify 'data' is the same as the built-indices data
    pub(crate) fn get<'a>(&'a self, key: &str, data: &'a str) -> Option<&'a str> {
        self.map.iter().find_map(|(k, v)| {
            if key == &data[k] {
                Some(&data[v])
            } else {
                None
            }
        })
    }
}

impl IntoOwned<'static> for TagIndices {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

mod helper {
    use std::cell::RefCell;

    #[derive(Default)]
    pub(crate) struct TagBuilder<'a> {
        buffer: RefCell<Vec<(&'a str, &'a str)>>,
    }

    impl<'a> TagBuilder<'a> {
        /// Build Buffer
        pub(crate) fn new() -> Self {
            Default::default()
        }
        /// Generate Tag Build buffer
        pub(crate) fn with(&'a self, key: &'a str, value: &'a str) -> &'a Self {
            // TODO: sanitise string slices?
            // TODO: should we be able to add multiple same key/values?
            self.buffer.borrow_mut().push((key, value));
            self
        }
        /// Naive approach to builder
        pub(crate) fn build(&'a self) -> String {
            // TODO: fix concat!
            let res: Vec<String> = self
                .buffer
                .borrow()
                .iter()
                .map(|n| format!("{}:{}", n.0, n.1))
                .collect();
            let output: String = res.join(",");
            format!("#{}", output)
        }
        pub(crate) fn count(&'a self) -> usize {
            self.buffer.borrow().len()
        }
    }

    #[test]
    fn test_same_slices() {
        let tag = TagBuilder::new();
        let r = tag.with("first", "second").with("first", "second").build();
        assert_eq!(r, "#first:second,first:second");
        assert_eq!(tag.count(), 2);
        assert_ne!(tag.count(), 1);
        {
            // TODO: should we be able to append new data after build?
            let r = tag.with("first", "second").build();
            assert_eq!(r, "#first:second,first:second,first:second");
            assert_eq!(tag.count(), 3);
            assert_ne!(tag.count(), 1);
        }
    }

    #[test]
    fn test_same_key_slices() {
        let tag = TagBuilder::new();
        let r = tag.with("first", "second").with("first", "value").build();
        assert_eq!(r, "#first:second,first:value");
        assert_eq!(tag.count(), 2);
        assert_ne!(tag.count(), 1);
    }

    #[test]
    fn test_not_sanitised() {
        let tag = TagBuilder::new();
        let r = tag.with("fir:st", "seco,nd").with("first", "value").build();
        assert_eq!(r, "#fir:st:seco,nd,first:value");
        assert_eq!(tag.count(), 2);
    }

    #[test]
    fn test_empty() {
        let tag = TagBuilder::new();
        let r = tag.build();
        assert_eq!(r, "#");
        // TODO: what would be a proper result for empty tag builder?
        assert_ne!(r, "");
        assert_eq!(tag.count(), 0);
    }
}
