use std::str::FromStr;
/// An error returned from the FromStr impls of RGB and Color
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorError {
    /// An invalid Hex string for `RGB`
    InvalidHexString,
    /// Unknown color name
    UnknownColor,
}

impl std::fmt::Display for ColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorError::InvalidHexString => write!(f, "invalid hex string"),
            ColorError::UnknownColor => write!(f, "unknown color"),
        }
    }
}

impl std::error::Error for ColorError {}

/// A 24-bit triplet for hex colors. Defaults to *White* `(0xFF,0xFF,0xFF)`
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RGB(pub u8, pub u8, pub u8);

impl RGB {
    /// Returns the `red` field
    pub fn red(self) -> u8 {
        self.0
    }

    /// Returns the `green` field
    pub fn green(self) -> u8 {
        self.1
    }

    /// Returns the `blue` field
    pub fn blue(self) -> u8 {
        self.2
    }
}

impl Default for RGB {
    /// Default color of `#FFFFFF` (White)
    fn default() -> Self {
        RGB(255, 255, 255)
    }
}

impl std::fmt::Display for RGB {
    /// Formats the RGB as `#RRGGBB` (in hex)
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b) = self;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RGB {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let RGB(r, g, b) = *self;
        let mut rgb = serializer.serialize_struct("rgb", 3)?;
        rgb.serialize_field("r", &r)?;
        rgb.serialize_field("g", &g)?;
        rgb.serialize_field("b", &b)?;
        rgb.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RGB {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Inner {
            r: u8,
            g: u8,
            b: u8,
        }
        let Inner { r, g, b } = Inner::deserialize(deserializer)?;
        Ok(Self(r, g, b))
    }
}

impl FromStr for RGB {
    type Err = ColorError;

    /// Tries to parse a string (`'#FFFFFF'` or `'FFFFFF'`) into RGB.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        let input = match (input.chars().next(), input.len()) {
            (Some('#'), 7) => &input[1..],
            (_, 6) => input,
            _ => return Err(ColorError::InvalidHexString),
        };

        u32::from_str_radix(&input, 16)
            .map(|s| {
                RGB(
                    ((s >> 16) & 0xFF) as u8,
                    ((s >> 8) & 0xFF) as u8,
                    (s & 0xFF) as u8,
                )
            })
            .map_err(|_| ColorError::InvalidHexString)
    }
}

/// Color represents a Twitch color
///
/// Twitch has named colors, and those with `Turbo` enabled accounts can set custom colors
///
/// A table of colors:
///
/// Name|Color|Alternative name *(can be parsed from name and these)*
/// ---|---|---
/// Blue |`#0000FF`| blue
/// BlueViolet |`#8A2BE2`| blueviolet, blue_violet, blue violet
/// CadetBlue |`#5F9EA0`| cadetblue, cadet_blue, cadet blue
/// Chocolate |`#D2691E`| chocolate
/// Coral |`#FF7F50`| coral
/// DodgerBlue |`#1E90FF`| dodgerblue, dodger_blue, dodger blue
/// Firebrick |`#B22222`| firebrick
/// GoldenRod |`#DAA520`| goldenrod, golden_rod, golden rod
/// Green |`#008000`| green
/// HotPink |`#FF69B4`| hotpink, hot_pink, hot pink
/// OrangeRed |`#FF4500`| orangered, orange_red, orange red
/// Red |`#FF0000`| red
/// SeaGreen |`#2E8B57`| seagreen, sea_green, sea green
/// SpringGreen |`#00FF7F`| springgreen, spring_green, spring green
/// YellowGreen |`#ADFF2F`| yellowgreen, yellow_green, yellow green
/// Turbo |*custom*| ---
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// The name of the Twitch color
    pub kind: TwitchColor,
    /// The RGB triplet for this color    
    pub rgb: RGB,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TwitchColor::*;
        let name = match self.kind {
            Blue => "Blue",
            BlueViolet => "BlueViolet",
            CadetBlue => "CadetBlue",
            Chocolate => "Chocolate",
            Coral => "Coral",
            DodgerBlue => "DodgerBlue",
            Firebrick => "Firebrick",
            GoldenRod => "GoldenRod",
            Green => "Green",
            HotPink => "HotPink",
            OrangeRed => "OrangeRed",
            Red => "Red",
            SeaGreen => "SeaGreen",
            SpringGreen => "SpringGreen",
            YellowGreen => "YellowGreen",
            Turbo => return write!(f, "{}", self.rgb.to_string()),
            __Nonexhaustive => unreachable!(),
        };
        write!(f, "{}", name)
    }
}

impl Default for Color {
    /// Defaults to having a kind of `TwitchColor::Turbo` and an RGB of `#FFFFFF`
    fn default() -> Self {
        Self {
            kind: TwitchColor::Turbo,
            rgb: RGB::default(),
        }
    }
}

impl FromStr for Color {
    type Err = ColorError;
    /// Tries to parse the twitch color name
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        use TwitchColor::*;

        let find = |color| {
            let colors = twitch_colors();
            colors[colors.iter().position(|(d, _)| *d == color).unwrap()]
        };

        let (kind, rgb) = match input {
            "Blue" | "blue" => find(Blue),
            "BlueViolet" | "blue_violet" | "blueviolet" | "blue violet" => find(BlueViolet),
            "CadetBlue" | "cadet_blue" | "cadetblue" | "cadet blue" => find(CadetBlue),
            "Chocolate" | "chocolate" => find(Chocolate),
            "Coral" | "coral" => find(Coral),
            "DodgerBlue" | "dodger_blue" | "dodgerblue" | "dodger blue" => find(DodgerBlue),
            "Firebrick" | "firebrick" => find(Firebrick),
            "GoldenRod" | "golden_rod" | "goldenrod" | "golden rod" => find(GoldenRod),
            "Green" | "green" => find(Green),
            "HotPink" | "hot_pink" | "hotpink" | "hot pink" => find(HotPink),
            "OrangeRed" | "orange_red" | "orangered" | "orange red" => find(OrangeRed),
            "Red" | "red" => find(Red),
            "SeaGreen" | "sea_green" | "seagreen" | "sea green" => find(SeaGreen),
            "SpringGreen" | "spring_green" | "springgreen" | "spring green" => find(SpringGreen),
            "YellowGreen" | "yellow_green" | "yellowgreen" | "yellow green" => find(YellowGreen),
            _ => (Turbo, input.parse::<RGB>()?),
        };

        Ok(Self { kind, rgb })
    }
}

/// These are the default Twitch colors
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TwitchColor {
    /// RGB (hex): #0000FF
    Blue,
    /// RGB (hex): #8A2BE2
    BlueViolet,
    /// RGB (hex): #5F9EA0
    CadetBlue,
    /// RGB (hex): #D2691E
    Chocolate,
    /// RGB (hex): #FF7F50
    Coral,
    /// RGB (hex): #1E90FF
    DodgerBlue,
    /// RGB (hex): #B22222
    Firebrick,
    /// RGB (hex): #DAA520
    GoldenRod,
    /// RGB (hex): #008000
    Green,
    /// RGB (hex): #FF69B4
    HotPink,
    /// RGB (hex): #FF4500
    OrangeRed,
    /// RGB (hex): #FF0000
    Red,
    /// RGB (hex): #2E8B57
    SeaGreen,
    /// RGB (hex): #00FF7F
    SpringGreen,
    /// RGB (hex): #ADFF2F
    YellowGreen,
    /// Turbo colors are custom user-selected colors
    Turbo,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<RGB> for Color {
    fn from(rgb: RGB) -> Self {
        Color {
            kind: rgb.into(),
            rgb,
        }
    }
}

impl From<Color> for RGB {
    fn from(color: Color) -> Self {
        color.rgb
    }
}

impl From<RGB> for TwitchColor {
    fn from(rgb: RGB) -> Self {
        twitch_colors()
            .iter()
            .find(|(_, color)| *color == rgb)
            .map(|&(c, _)| c)
            .unwrap_or_else(|| TwitchColor::Turbo)
    }
}

impl From<TwitchColor> for RGB {
    fn from(color: TwitchColor) -> Self {
        twitch_colors()
            .iter()
            .find(|(c, _)| *c == color)
            .map(|&(_, r)| r)
            .unwrap_or_default()
    }
}

/// A helper method that returns a const array of [`TwitchColor`](./enum.TwitchColor.html) colors to [`RGB`](./struct.RGB.html)
pub const fn twitch_colors() -> [(TwitchColor, RGB); 15] {
    use TwitchColor::*;
    [
        (Blue, RGB(0x00, 0x00, 0xFF)),
        (BlueViolet, RGB(0x8A, 0x2B, 0xE2)),
        (CadetBlue, RGB(0x5F, 0x9E, 0xA0)),
        (Chocolate, RGB(0xD2, 0x69, 0x1E)),
        (Coral, RGB(0xFF, 0x7F, 0x50)),
        (DodgerBlue, RGB(0x1E, 0x90, 0xFF)),
        (Firebrick, RGB(0xB2, 0x22, 0x22)),
        (GoldenRod, RGB(0xDA, 0xA5, 0x20)),
        (Green, RGB(0x00, 0x80, 0x00)),
        (HotPink, RGB(0xFF, 0x69, 0xB4)),
        (OrangeRed, RGB(0xFF, 0x45, 0x00)),
        (Red, RGB(0xFF, 0x00, 0x00)),
        (SeaGreen, RGB(0x2E, 0x8B, 0x57)),
        (SpringGreen, RGB(0x00, 0xFF, 0x7F)),
        (YellowGreen, RGB(0xAD, 0xFF, 0x2F)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_color() {
        let color: Color = "blue".parse().unwrap();
        assert_eq!(color.kind, TwitchColor::Blue);
        assert_eq!(color.rgb, RGB(0, 0, 255));
        assert_eq!(color.to_string(), "Blue");
    }

    #[test]
    fn parse_turbo_color() {
        let color: Color = "#FAFAFA".parse().unwrap();
        assert_eq!(color.kind, TwitchColor::Turbo);
        assert_eq!(color.rgb, RGB(250, 250, 250));
        assert_eq!(color.to_string(), "#FAFAFA");

        let color: Color = "FAFAFA".parse().unwrap();
        assert_eq!(color.kind, TwitchColor::Turbo);
        assert_eq!(color.rgb, RGB(250, 250, 250));
        assert_eq!(color.to_string(), "#FAFAFA");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn round_trip_color() {
        let color: Color = "blue".parse().unwrap();
        let json = serde_json::to_string(&color).unwrap();
        let parsed: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, color);
        assert_eq!(color.kind, TwitchColor::Blue);
        assert_eq!(color.rgb, RGB(0, 0, 255));
        assert_eq!(color.to_string(), "Blue");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn round_trip_turbo_color() {
        let color: Color = "#FAFAFA".parse().unwrap();
        let json = serde_json::to_string(&color).unwrap();
        let parsed: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, color);
        assert_eq!(color.kind, TwitchColor::Turbo);
        assert_eq!(color.rgb, RGB(250, 250, 250));
        assert_eq!(color.to_string(), "#FAFAFA");

        let color: Color = "FAFAFA".parse().unwrap();
        let json = serde_json::to_string(&color).unwrap();
        let parsed: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, color);
        assert_eq!(color.kind, TwitchColor::Turbo);
        assert_eq!(color.rgb, RGB(250, 250, 250));
        assert_eq!(color.to_string(), "#FAFAFA");
    }
}
