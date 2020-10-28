use crate::{
    commands::{self, types::Reply},
    messages::Privmsg,
    Encodable,
};

use std::io::{Error, ErrorKind, Write};

/// Extensions to the `Privmsg` message type
pub trait PrivmsgExt {
    /// Reply to this message with `data`
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()>;

    /// Send a message back to the channel this Privmsg came from
    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()>;
}

impl<W> PrivmsgExt for W
where
    W: Write + Sized,
{
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        make_reply(msg, data)?.encode(self)?;
        self.flush()
    }

    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        commands::privmsg(msg.channel(), data).encode(self)?;
        self.flush()
    }
}

fn make_reply<'a>(msg: &'a Privmsg<'_>, data: &'a str) -> std::io::Result<Reply<'a>> {
    Ok(commands::reply(
        msg.channel(),
        msg.tags().get("id").ok_or_else(|| {
            Error::new(ErrorKind::PermissionDenied, "you must have `TAGS` enabled")
        })?,
        data,
    ))
}

#[cfg(feature = "writer")]
#[cfg_attr(docsrs, doc(cfg(feature = "writer")))]
impl PrivmsgExt for crate::writer::MpscWriter {
    fn reply(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        self.send(make_reply(msg, data)?)
    }

    fn say(&mut self, msg: &Privmsg<'_>, data: &str) -> std::io::Result<()> {
        self.send(commands::privmsg(msg.channel(), data))
    }
}
