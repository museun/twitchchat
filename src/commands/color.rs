use super::Encodable;
use std::convert::TryInto;
use std::io::{Result, Write};

/// Change your username color.
#[non_exhaustive]
#[must_use = "commands must be encoded"]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Color<'a> {
    pub(crate) color: crate::twitch::Color,
    #[cfg_attr(feature = "serde", serde(skip))]
    marker: std::marker::PhantomData<&'a ()>,
}

/// Change your username `color`.
// TODO documentation tests to show how you can use this with different types
pub fn color<T>(color: T) -> std::result::Result<Color<'static>, T::Error>
where
    T: TryInto<crate::twitch::Color>,
{
    color.try_into().map(|color| Color {
        color,
        marker: std::marker::PhantomData,
    })
}

impl<'a> Encodable for Color<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_jtv_cmd!(buf, "/color {}", &self.color.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn color_encode() {
        let blue: crate::twitch::Color = "blue".parse().unwrap();
        test_encode(
            color(blue).unwrap(),
            format!("PRIVMSG jtv :/color {}\r\n", blue),
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn color_serde() {
        let blue: crate::twitch::Color = "blue".parse().unwrap();
        test_serde(
            color(blue).unwrap(),
            format!("PRIVMSG jtv :/color {}\r\n", blue),
        )
    }
}
