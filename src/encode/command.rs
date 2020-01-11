use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Command<'a> {
    pub(crate) data: &'a str,
}

impl<'a> Encodable for Command<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_fmt(format_args!("PRIVMSG jtv :{}\r\n", self.data))
    }
}

pub fn command<'a>(data: &'a str) -> Command<'a> {
    Command { data }
}
