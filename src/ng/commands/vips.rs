use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]


#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Vips<'a> {
    pub 
    channel: &'a str,
}

pub fn vips(channel: &str) -> Vips<'_> {
    Vips { channel }
}

impl<'a> Encodable for Vips<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/vips"])
    }
}
