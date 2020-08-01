use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Commercial<'a> {
    pub channel: &'a str,
    pub length: Option<usize>,
}

pub fn commercial(channel: &str, length: impl Into<Option<usize>>) -> Commercial<'_> {
    Commercial {
        channel,
        length: length.into(),
    }
}

impl<'a> Encodable for Commercial<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            &self.channel,
            &[
                &"/commercial",
                &self
                    .length
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or_default(),
            ],
        )
    }
}
