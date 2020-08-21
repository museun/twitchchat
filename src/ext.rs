use crate::{messages::Privmsg, Encodable};
use std::io::Write;

/// Extensions to the `Privmsg` message type
pub trait PrivmsgExt {
    /// Reply to this message with `data` over `writer`
    fn reply<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
    where
        W: Write + ?Sized;

    /// Send a message back to the channel this Privmsg came from
    fn say<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
    where
        W: Write + ?Sized;
}

impl<'a> PrivmsgExt for Privmsg<'a> {
    fn reply<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
    where
        W: Write + ?Sized,
    {
        let cmd = crate::commands::reply(
            self.channel(),
            self.tags().get("id").ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "you must have `TAGS` enabled",
                )
            })?,
            data,
        );
        cmd.encode(writer)?;
        writer.flush()
    }

    fn say<W>(&self, writer: &mut W, data: &str) -> std::io::Result<()>
    where
        W: Write + ?Sized,
    {
        let cmd = crate::commands::privmsg(self.channel(), data);
        cmd.encode(writer)?;
        writer.flush()
    }
}
