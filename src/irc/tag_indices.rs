use std::borrow::Cow;

use crate::{maybe_owned::MaybeOwned, IntoOwned, MessageError};

/// Pre-computed tag indices
///
/// This type is only exposed for those wanting to extend/make custom types.
#[derive(Default, Clone, PartialEq)]
pub struct TagIndices {
    // NOTE this is a hack to keep the semver stable, in v0.15 this'll go back to being borrowed.
    pub(super) map: Box<[(Cow<'static, str>, Cow<'static, str>)]>,
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
    pub fn build_indices(input: &str) -> Result<Self, MessageError> {
        if !input.starts_with('@') {
            return Ok(Self::default());
        }

        input[1..]
            .split_terminator(';')
            .enumerate()
            .map(|(pos, input)| {
                use MessageError::{MissingTagKey, MissingTagValue};
                let expect = |s: Option<&str>, err: fn(usize) -> MessageError| {
                    s.map(ToString::to_string)
                        .map(Cow::from)
                        .ok_or_else(|| err(pos))
                };

                let mut iter = input.split('=');
                let key = expect(iter.next().filter(|s| !s.is_empty()), MissingTagKey)?;
                let value = expect(iter.next(), MissingTagValue)?;
                Ok((key, value))
            })
            .collect::<Result<_, _>>()
            .map(|map| Self { map })
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
    pub(crate) fn get_unescaped<'a>(&'a self, key: &str) -> Option<MaybeOwned<'a>> {
        self.get(key).map(crate::irc::tags::unescape_str)
    }

    // NOTE: this isn't public because they don't verify 'data' is the same as the built-indices data
    pub(crate) fn get<'a>(&'a self, key: &str) -> Option<&'a str> {
        let key = crate::irc::tags::escape_str(key);
        self.map
            .iter()
            .find_map(|(k, v)| if &key == k { Some(&**v) } else { None })
    }
}

impl IntoOwned<'static> for TagIndices {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf8_tags() {
        let input = "@id=86293428;login=yuebing233;display_name=月饼;foo=bar";
        TagIndices::build_indices(input).unwrap();
    }
}
