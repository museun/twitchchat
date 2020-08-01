use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]


#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SlowOff<'a> {
    pub 
    channel: &'a str,
}

pub fn slow_off(channel: &str) -> SlowOff<'_> {
    SlowOff { channel }
}

impl<'a> Encodable for SlowOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/slowoff"])
    }
}
