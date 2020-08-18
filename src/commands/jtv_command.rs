use crate::Encodable;
use std::io::{Result, Write};

/// Sends the data as a command to the 'jtv' channel (e.g. `/color #FFFFFF`)
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct JtvCommand<'a> {
    pub(crate) data: &'a str,
}

/// Sends the data as a command to the 'jtv' channel (e.g. `/color #FFFFFF`)
pub const fn jtv_command(data: &str) -> JtvCommand<'_> {
    JtvCommand { data }
}

impl<'a> Encodable for JtvCommand<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_jtv_cmd!(buf, self.data)
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
