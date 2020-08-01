use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Privmsg<'a> {
    pub 
    channel: &'a str,
    pub 
    data: &'a str,
}

pub fn privmsg<'a>(channel: &'a str, data: &'a str) -> Privmsg<'a> {
    Privmsg { channel, data }
}

impl<'a> Encodable for Privmsg<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts_term(&[&"PRIVMSG ", &self.channel, &" :", &self.data])
    }
}
