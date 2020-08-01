use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Me<'a> {
    pub 
    channel: &'a str,
    pub 
    msg: &'a str,
}

pub fn me<'a>(channel: &'a str, msg: &'a str) -> Me<'a> {
    Me { channel, msg }
}

impl<'a> Encodable for Me<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/me", &self.msg])
    }
}
