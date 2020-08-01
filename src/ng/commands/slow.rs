use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Slow<'a> {
    pub channel: &'a str,
    pub duration: usize,
}

pub fn slow(channel: &str, duration: impl Into<Option<usize>>) -> Slow<'_> {
    Slow {
        channel,
        duration: duration.into().unwrap_or(120),
    }
}

impl<'a> Encodable for Slow<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/slow", &self.duration.to_string()])
    }
}
