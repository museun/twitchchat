use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Marker<'a> {
    pub(crate) comment: Option<&'a str>,
}

impl<'a> Encodable for Marker<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self.comment {
            Some(comment) => command(&format!("/marker {}", comment)).encode(writer),
            None => command("/marker").encode(writer),
        }
    }
}

// TODO limit this to 140
pub fn marker<'a>(comment: impl Into<Option<&'a str>>) -> Marker<'a> {
    Marker {
        comment: comment.into(),
    }
}
