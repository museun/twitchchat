use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct R9kBetaOff<'a> {
    pub 
    channel: &'a str,
}

pub fn r9k_beta_off(channel: &str) -> R9kBetaOff<'_> {
    R9kBetaOff { channel }
}

impl<'a> Encodable for R9kBetaOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/r9kbetaoff"])
    }
}
