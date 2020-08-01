use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Ban<'a> {
    pub channel: &'a str,
    pub username: &'a str,
    pub reason: Option<&'a str>,
}

impl<'a> Encodable for Ban<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            self.channel,
            &[&"/ban", &self.username, &self.reason.unwrap_or_default()],
        )
    }
}

pub fn ban<'a>(channel: &'a str, username: &'a str, reason: impl Into<Option<&'a str>>) -> Ban<'a> {
    Ban {
        channel,
        username,
        reason: reason.into(),
    }
}
