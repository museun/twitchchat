use super::*;

/// A Part message. Used to leave a channel
#[derive(Copy, Clone, Debug)]
pub struct Part<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> Encodable for Part<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_fmt(format_args!("PART {}\r\n", self.channel))
    }
}

/// Leave a channel
pub fn part(channel: &str) -> Part<'_> {
    Part { channel }
}
