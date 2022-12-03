use std::ops::Range;

use crate::maybe_owned::MaybeOwned;

/**
Emotes are little pictograms used in-line in Twitch messages

They are presented (to the irc connection) in a `id:range1,range2/id2:range1,..` form which marks the byte position that the emote is located.

# example:
`"testing Kappa"` would be `25:8-13`

`"Kappa testing Kappa"` would be `25:0-5,14-19`
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Emotes<'a> {
    /// This emote id, e.g. `Kappa = 25`
    pub id: MaybeOwned<'a>,
    /// A list of [Range] in the message where this emote is found
    ///
    /// [Range]: https://doc.rust-lang.org/std/ops/struct.Range.html
    pub ranges: Vec<Range<u16>>,
}

impl<'a> Emotes<'a> {
    /// Parse emotes from a string, returning an iterator over each emote
    pub fn parse(input: &'a str) -> impl Iterator<Item = Emotes<'a>> + 'a {
        input.split_terminator('/').filter_map(Self::parse_item)
    }

    /// Parse single emote
    pub fn parse_item(item: &'a str) -> Option<Self> {
        get_parts(item, ':').and_then(|(head, tail)| {
            let emotes = Self {
                id: head.into(),
                ranges: get_ranges(tail).collect(),
            };
            emotes.into()
        })
    }
}

impl<'a> crate::IntoOwned<'a> for Emotes<'a> {
    type Output = Emotes<'static>;
    fn into_owned(self) -> Self::Output {
        Emotes {
            id: self.id.into_owned(),
            ranges: self.ranges,
        }
    }
}

#[inline]
fn get_parts(input: &str, sep: char) -> Option<(&str, &str)> {
    let mut split = input.split_terminator(sep);
    (split.next()?, split.next()?).into()
}

#[inline]
fn get_ranges(tail: &str) -> impl Iterator<Item = Range<u16>> + '_ {
    tail.split_terminator(',')
        .filter_map(|s| get_parts(s, '-'))
        .filter_map(move |(start, end)| {
            let (start, end) = (start.parse().ok()?, end.parse().ok()?);
            Range { start, end }.into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        macro_rules! emote {
            ($id:expr, $($r:expr),* $(,)?) => {
                Emotes {
                    id: $id.to_string().into(),
                    ranges: vec![$($r),*]
                }
            };

        }

        let inputs = &[
            (
                "25:0-4,6-10,12-16",
                vec![emote!(25, (0..4), (6..10), (12..16))],
            ),
            (
                "25:0-4", //
                vec![emote!(25, (0..4))],
            ),
            (
                "1077966:0-6/25:8-12",
                vec![emote!(1_077_966, (0..6)), emote!(25, (8..12))],
            ),
            (
                "25:0-4,6-10/33:12-19",
                vec![emote!(25, (0..4), (6..10)), emote!(33, (12..19))],
            ),
            (
                "25:0-4,15-19/33:6-13",
                vec![emote!(25, (0..4), (15..19)), emote!(33, (6..13))],
            ),
            (
                "33:0-7/25:9-13,15-19",
                vec![emote!(33, (0..7)), emote!(25, (9..13), (15..19))],
            ),
            (
                "emotesv2_4c3b4ed516de493bbcd2df2f5d450f49:0-25",
                vec![emote!("emotesv2_4c3b4ed516de493bbcd2df2f5d450f49", (0..25))],
            ),
        ];

        for (input, expect) in inputs {
            let emotes = Emotes::parse(input).collect::<Vec<_>>();
            assert_eq!(emotes.len(), expect.len());
            assert_eq!(emotes, *expect);
        }
    }
}
