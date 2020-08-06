use crate::Encodable;
use std::convert::TryInto;
use std::io::{Result, Write};

use super::ByteWriter;

/// Change your username color.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Color<'a> {
    pub(crate) color: crate::color::Color,
    #[cfg_attr(feature = "serde", serde(skip))]
    marker: std::marker::PhantomData<&'a ()>,
}

/// Change your username color.
pub fn color<T>(color: T) -> std::result::Result<Color<'static>, T::Error>
where
    T: TryInto<crate::color::Color>,
{
    color.try_into().map(|color| Color {
        color,
        marker: std::marker::PhantomData,
    })
}

impl<'a> Encodable for Color<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/color", &self.color.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn color_encode() {
        let blue: crate::color::Color = "blue".parse().unwrap();
        test_encode(
            color(blue).unwrap(),
            format!("PRIVMSG jtv :/color {}\r\n", blue),
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn color_serde() {
        let blue: crate::color::Color = "blue".parse().unwrap();
        test_serde(
            color(blue).unwrap(),
            format!("PRIVMSG jtv :/color {}\r\n", blue),
        )
    }
}
