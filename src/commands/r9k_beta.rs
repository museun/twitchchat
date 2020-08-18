use super::Channel;
use crate::Encodable;
use std::io::{Result, Write};

/// Enables r9k mode.    
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct R9kBeta<'a> {
    pub(crate) channel: &'a str,
}

/// Enables r9k mode.
///
/// Use [r9k_beta_off] to disable.
///
/// [r9k_beta_off]: ./fn.r9k_beta_off.html
pub const fn r9k_beta(channel: &str) -> R9kBeta<'_> {
    R9kBeta { channel }
}

impl<'a> Encodable for R9kBeta<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(&self.channel) => "/r9kbeta")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn r9k_beta_encode() {
        test_encode(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n");
    }

    #[test]
    fn r9k_beta_ensure_channel_encode() {
        test_encode(r9k_beta("museun"), "PRIVMSG #museun :/r9kbeta\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn r9k_beta_serde() {
        test_serde(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn r9k_beta_ensure_channel_serde() {
        test_serde(r9k_beta("museun"), "PRIVMSG #museun :/r9kbeta\r\n");
    }
}
