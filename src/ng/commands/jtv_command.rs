use crate::ng::Encodable;
use std::io::{Result, Write};

use super::ByteWriter;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct JtvCommand<'a> {
    pub data: &'a str,
}

pub fn jtv_command(data: &str) -> JtvCommand<'_> {
    JtvCommand { data }
}

impl<'a> Encodable for JtvCommand<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&self.data])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn jtv_command_encode() {
        test_encode(jtv_command("/help"), "PRIVMSG jtv :/help\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn jtv_command_serde() {
        test_serde(jtv_command("/help"), "PRIVMSG jtv :/help\r\n");
    }
}
