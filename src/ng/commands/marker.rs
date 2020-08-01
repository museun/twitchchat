use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Marker<'a> {
    pub 
    channel: &'a str,
    pub 
    comment: Option<&'a str>,
}

pub fn marker<'a>(channel: &'a str, comment: impl Into<Option<&'a str>>) -> Marker<'_> {
    Marker {
        channel,
        comment: comment.into(),
    }
}

impl<'a> Encodable for Marker<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        fn truncate(s: &str) -> &str {
            const MAX: usize = 140;
            if s.len() <= MAX {
                return s;
            }

            for n in (0..=MAX).rev() {
                if s.is_char_boundary(n) {
                    return &s[..n];
                }
            }

            ""
        }

        ByteWriter::new(buf).command(
            self.channel,
            &[&"/marker", &self.comment.map(truncate).unwrap_or_default()],
        )
    }
}
