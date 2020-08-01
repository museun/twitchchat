use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Part<'a> {
    pub 
    channel: &'a str,
}

pub fn part(channel: &str) -> Part<'_> {
    Part { channel }
}

impl<'a> Encodable for Part<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts(&[&"PART", &self.channel])
    }
}
