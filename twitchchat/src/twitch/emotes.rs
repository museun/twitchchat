use std::ops::Range;

/**
Emotes are little pictograms used in-line in Twitch messages

They are presented (to the irc connection) in a `id:range1,range2/id2:range1,..` form which marks the byte position that the emote is located.

# example:
`"testing Kappa"` would be `25:8-13`

`"Kappa testing Kappa"` would be `25:0-5,14-19`
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Emotes {
    /// This emote id, e.g. `Kappa = 25`    
    pub id: usize,
    /// A list of [`std::ops::Range`] in the message where this emote is found
    pub ranges: Vec<Range<u16>>,
}

impl Emotes {
    /// Parse emotes from a string, returning an iterator over each emote
    pub fn parse(input: &str) -> impl Iterator<Item = Self> + '_ {
        input.split_terminator('/').filter_map(Self::parse_item)
    }

    /// Parse single emote
    pub fn parse_item(item: &str) -> Option<Self> {
        get_parts(item, ':').and_then(|(head, tail)| {
            let emotes = Self {
                id: head.parse().ok()?,
                ranges: get_ranges(tail).collect(),
            };
            emotes.into()
        })
    }
}

fn get_parts(input: &str, sep: char) -> Option<(&str, &str)> {
    let mut split = input.split_terminator(sep);
    (split.next()?, split.next()?).into()
}

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
                    id: $id,
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
        ];

        for (input, expect) in inputs {
            let emotes = Emotes::parse(input).collect::<Vec<_>>();
            assert_eq!(emotes.len(), expect.len());
            assert_eq!(emotes, *expect);
        }
    }
}
