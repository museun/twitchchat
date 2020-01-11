use super::*;

/// A Pong message. Used to respond to a heartbeat
#[derive(Copy, Clone, Debug)]
pub struct Pong<'a> {
    pub(crate) token: &'a str,
}

impl<'a> Encodable for Pong<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_fmt(format_args!("PONG :{}\r\n", self.token))
    }
}

/// Response to a heartbeat
pub fn pong(token: &str) -> Pong<'_> {
    Pong { token }
}
