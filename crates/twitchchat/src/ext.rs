use crate::{commands, messages::Privmsg, Encodable};
use std::io::{Error, ErrorKind, Write};

/// Extensions to the `Privmsg` message type
pub trait PrivmsgExt {
    /// Reply to this message with `data`
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()>;

    /// Send a message back to the channel this Privmsg came from
    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()>;
}

#[cfg(feature = "write")]
impl PrivmsgExt for crate::writer::MpscWriter {
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        let cmd = commands::reply(
            msg.channel(),
            msg.tags().get("id").ok_or_else(|| {
                Error::new(ErrorKind::PermissionDenied, "you must have `TAGS` enabled")
            })?,
            data,
        );
        self.send(cmd)
    }

    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        self.send(commands::privmsg(msg.channel(), data))
    }
}

impl<W> PrivmsgExt for W
where
    W: Write + Sized,
{
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        let cmd = commands::reply(
            msg.channel(),
            msg.tags().get("id").ok_or_else(|| {
                Error::new(ErrorKind::PermissionDenied, "you must have `TAGS` enabled")
            })?,
            data,
        );
        cmd.encode(self)?;
        self.flush()
    }

    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        commands::privmsg(msg.channel(), data).encode(self)?;
        self.flush()
    }
}
