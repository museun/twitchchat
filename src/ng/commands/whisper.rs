use crate::ng::Encodable;
use std::{

    io::{Result, Write},
};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]


#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Whisper<'a> {
    pub 
    username: &'a str,
    pub 
    message: &'a str,
}

pub fn whisper<'a>(username: &'a str, message: &'a str) -> Whisper<'a> {
    Whisper { username, message }
}

impl<'a> Encodable for Whisper<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/w", &self.username, &self.message])
    }
}
