use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Host<'a> {
    pub source: &'a str,
    pub target: &'a str,
}

pub fn host<'a>(source: &'a str, target: &'a str) -> Host<'a> {
    Host { source, target }
}

impl<'a> Encodable for Host<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.source, &[&"/host", &self.target])
    }
}
