use super::Encodable;
use std::io::{Result, Write};

/// Send a raw IRC-style message
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Raw<'a> {
    pub(crate) data: &'a str,
}

/// Send a raw IRC-style message. This appends a `\r\n` for you.
pub const fn raw(data: &str) -> Raw<'_> {
    Raw { data }
}

impl<'a> Encodable for Raw<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_nl!(buf, "{}", self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn raw_encode() {
        test_encode(
            raw("PRIVMSG #test :this is a test"),
            "PRIVMSG #test :this is a test\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn raw_serde() {
        test_serde(
            raw("PRIVMSG #test :this is a test"),
            "PRIVMSG #test :this is a test\r\n",
        );
    }
}
