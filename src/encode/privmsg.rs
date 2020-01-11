use super::*;

/// A Privmsg message. Used to send data to a target
#[derive(Copy, Clone, Debug)]
pub struct Privmsg<'a> {
    pub(crate) target: &'a str,
    pub(crate) data: &'a str,
}

impl<'a> Encodable for Privmsg<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_fmt(format_args!("PRIVMSG {} :{}\r\n", self.target, self.data))
    }
}

/// Send data to a target
pub fn privmsg<'a>(target: &'a str, data: &'a str) -> Privmsg<'a> {
    Privmsg { target, data }
}
