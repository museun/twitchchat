use super::*;
use crate::color::Color as TwitchColor;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub(crate) color: TwitchColor,
}

impl Encodable for Color {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/color {}", self.color)).encode(writer)
    }
}

pub fn color(color: TwitchColor) -> Color {
    Color { color }
}
