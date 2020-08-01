use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Raid<'a> {
    pub 
    source: &'a str,
    pub 
    target: &'a str,
}

pub fn raid<'a>(source: &'a str, target: &'a str) -> Raid<'a> {
    Raid { source, target }
}

impl<'a> Encodable for Raid<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.source, &[&"/raid", &self.target])
    }
}
