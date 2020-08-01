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

pub fn disconnect() -> Disconnect<'static> {
    Disconnect {
        marker: std::marker::PhantomData,
    }
}

impl<'a> Encodable for Disconnect<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/disconnect"])
    }
}
