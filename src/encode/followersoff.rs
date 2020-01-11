use super::*;

#[derive(Copy, Clone, Debug)]
pub struct FollowersOff {}

impl Encodable for FollowersOff {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/followersoff").encode(writer)
    }
}

pub fn followers_off() -> FollowersOff {
    FollowersOff {}
}
