use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Disconnect<'a> {
    #[cfg_attr(feature = "serde", serde(skip))]
    marker: std::marker::PhantomData<&'a Disconnect<'a>>,
}

impl<'a> Disconnect<'a> {
    pub const fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

pub fn disconnect() -> Disconnect<'static> {
    Disconnect::new()
}

impl<'a> Encodable for Disconnect<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/disconnect"])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn disconnect_encode() {
        test_encode(disconnect(), "PRIVMSG jtv :/disconnect\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn disconnect_serde() {
        test_serde(disconnect(), "PRIVMSG jtv :/disconnect\r\n")
    }
}
