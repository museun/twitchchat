use std::ops::Range;

/// Emotes are little pictograms used inline in twitch messages
///
/// They are presented (to the irc connection) in a
/// `id1:range1,range2/id2:range1,..` form which marks the byte position that
/// the emote is at.
///
/// Examples:
///
/// `"testing Kappa"` would be `25:8-13`
/// `"Kappa testing Kappa"` would be `25:0-5,14-19`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Emotes {
    /// The emote id, e.g. `Kappa = 25`
    pub id: usize,
    /// A list of [`Range`](https://doc.rust-lang.org/std/ops/struct.Range.html) in the message where this emote is
    /// found
    pub ranges: Vec<Range<u16>>,
}

impl Emotes {
    pub(in crate) fn parse<'a>(input: &'a str) -> impl Iterator<Item = Self> + 'a {
        input
            .split_terminator('/')
            .filter_map(|s| Self::get_parts(s, ':'))
            .filter_map(|(head, tail)| {
                Some(Self {
                    id: head.parse().ok()?,
                    ranges: Self::get_ranges(&tail).collect(),
                })
            })
    }

    #[inline]
    fn get_ranges<'a>(tail: &'a str) -> impl Iterator<Item = Range<u16>> + 'a {
        tail.split_terminator(',')
            .map(|s| Self::get_parts(s, '-'))
            .filter_map(move |parts| {
                let (start, end) = parts?;
                let (start, end) = (start.parse().ok()?, end.parse().ok()?);
                Some(Range { start, end })
            })
    }

    #[inline(always)]
    fn get_parts(input: &str, sep: char) -> Option<(&str, &str)> {
        let mut s = input.split_terminator(sep);
        Some((s.next()?, s.next()?))
    }
}

// impl Emotes {
//     // TODO look up the emote id https://twitchemotes.com/emotes/{id}
//     // https://static-cdn.jtvnw.net/emoticons/v1/{id}/3.0
//     pub fn lookup(&self) -> String {
//         unimplemented!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn parse_emotes() {
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
                vec![
                    emote!(25, (0..4), (6..10), (12..16)), //
                ],
            ),
            (
                "25:0-4",
                vec![
                    emote!(25, (0..4)), //
                ],
            ),
            (
                "1077966:0-6/25:8-12",
                vec![
                    emote!(1077966, (0..6)), //
                    emote!(25, (8..12)),     //
                ],
            ),
            (
                "25:0-4,6-10/33:12-19",
                vec![
                    emote!(25, (0..4), (6..10)), //
                    emote!(33, (12..19)),        //
                ],
            ),
            (
                "25:0-4,15-19/33:6-13",
                vec![
                    emote!(25, (0..4), (15..19)), //
                    emote!(33, (6..13)),          //
                ],
            ),
            (
                "33:0-7/25:9-13,15-19",
                vec![
                    emote!(33, (0..7)),            //
                    emote!(25, (9..13), (15..19)), //
                ],
            ),
        ];

        for (input, expect) in inputs {
            let emotes = Emotes::parse(&input).collect::<Vec<_>>();
            assert_eq!(emotes.len(), expect.len());
            assert_eq!(emotes, *expect);
        }
    }
}
