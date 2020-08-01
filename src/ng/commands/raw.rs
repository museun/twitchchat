use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Raw<'a> {
    pub data: &'a str,
}

pub fn raw(data: &str) -> Raw<'_> {
    Raw { data }
}

impl<'a> Encodable for Raw<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).write_bytes(self.data)
    }
}
