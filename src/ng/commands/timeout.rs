use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]


#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Timeout<'a> {
    pub 
    channel: &'a str,
    pub 
    username: &'a str,
    pub 
    duration: Option<&'a str>,
    pub 
    reason: Option<&'a str>,
}

pub fn timeout<'a>(
    channel: &'a str,
    username: &'a str,
    duration: impl Into<Option<&'a str>>,
    reason: impl Into<Option<&'a str>>,
) -> Timeout<'a> {
    Timeout {
        channel,
        username,
        duration: duration.into(),
        reason: reason.into(),
    }
}

impl<'a> Encodable for Timeout<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            self.channel,
            &[
                &"/timeout",
                &self.username,
                &self.duration.unwrap_or_default(),
                &self.reason.unwrap_or_default(),
            ],
        )
    }
}
