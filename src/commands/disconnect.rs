use crate::Encodable;
use std::io::{Result, Write};

/// Reconnects to chat.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Disconnect<'a> {
    #[cfg_attr(feature = "serde", serde(skip))]
    marker: std::marker::PhantomData<&'a Disconnect<'a>>,
}

/// Reconnects to chat.
pub const fn disconnect() -> Disconnect<'static> {
    Disconnect {
        marker: std::marker::PhantomData,
    }
}

impl<'a> Encodable for Disconnect<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_jtv_cmd!(buf, "/disconnect")
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
