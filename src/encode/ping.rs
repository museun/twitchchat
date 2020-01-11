use super::*;

/// A Ping message. Used to request a heartbeat
#[derive(Copy, Clone, Debug)]
pub struct Ping<'a> {
    pub(crate) token: &'a str,
}

impl<'a> Encodable for Ping<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_fmt(format_args!("PING {}\r\n", self.token))
    }
}

/// Request a heartbeat
pub fn ping(token: &str) -> Ping<'_> {
    Ping { token }
}
