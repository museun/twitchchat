use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Followers<'a> {
    duration: &'a str,
}

impl<'a> Encodable for Followers<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/followers {}", self.duration)).encode(writer)
    }
}

// TODO use `time` here
pub fn followers(duration: &str) -> Followers<'_> {
    Followers { duration }
}
