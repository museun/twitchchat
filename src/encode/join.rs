use super::*;

/// A Join message. Used to join a channel
#[derive(Copy, Clone, Debug)]
pub struct Join<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> Encodable for Join<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_fmt(format_args!("JOIN {}\r\n", self.channel))
    }
}

/// Join a channel
pub fn join(channel: &str) -> Join<'_> {
    Join { channel }
}
