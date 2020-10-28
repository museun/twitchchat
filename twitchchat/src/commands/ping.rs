use super::Encodable;
use std::io::{Result, Write};

/// Request a servver response  with the provided token
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Ping<'a> {
    pub(crate) token: &'a str,
}

/// Request a servver response  with the provided token
pub const fn ping(token: &str) -> Ping<'_> {
    Ping { token }
}

impl<'a> Encodable for Ping<'a> {
    fn encode(&self, buf: &mut dyn Write) -> Result<()> {
        write_nl!(buf, "PING {}", self.token)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn ping_encode() {
        test_encode(ping("123456789"), "PING 123456789\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn ping_serde() {
        test_serde(ping("123456789"), "PING 123456789\r\n");
    }
}
