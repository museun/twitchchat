use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct SubscribersOff<'a> {
    pub 
    channel: &'a str,
}

pub fn subscribers_off(channel: &str) -> SubscribersOff<'_> {
    SubscribersOff { channel }
}

impl<'a> Encodable for SubscribersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/subscribersoff"])
    }
}
