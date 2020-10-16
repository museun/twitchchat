use crate::{messages::Privmsg, Encodable};
use std::io::Write;

/// Extensions to the `Privmsg` message type
pub trait PrivmsgExt {
    /// Reply to this message with `data`
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()>;

    /// Send a message back to the channel this Privmsg came from
    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()>;
}

impl<'a, W> PrivmsgExt for W
where
    W: Write + Sized,
{
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        let cmd = crate::commands::reply(
            msg.channel(),
            msg.tags().get("id").ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "you must have `TAGS` enabled",
                )
            })?,
            data,
        );
        cmd.encode(self)?;
        self.flush()
    }

    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        let cmd = crate::commands::privmsg(msg.channel(), data);
        cmd.encode(self)?;
        self.flush()
    }
}
