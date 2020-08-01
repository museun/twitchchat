use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Ping<'a> {
    pub 
    token: &'a str,
}

pub fn ping(token: &str) -> Ping<'_> {
    Ping { token }
}

impl<'a> Encodable for Ping<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts(&[&"PING", &self.token])
    }
}
