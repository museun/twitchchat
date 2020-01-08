/*!
Color types associated with Twitch.

Both an RGB triplet and named Colors are provided.

# RGB
## Try to parse a RGB from a #RRGGBB or RRGGBB
```
# use std::str::FromStr as _;
# use twitchchat::color::RGB;
let rgb: RGB = "#00FF19".parse().unwrap();
assert_eq!(rgb.red(), 0x00);
assert_eq!(rgb.green(), 0xFF);
assert_eq!(rgb.blue(), 0x19);

// or without the leading octothrope
let rgb: RGB = "00FF19".parse().unwrap();
assert_eq!(rgb.red(), 0x00);
assert_eq!(rgb.green(), 0xFF);
assert_eq!(rgb.blue(), 0x19);
```

## Turning it back into a string
```
# use std::str::FromStr as _;
# use twitchchat::color::RGB;
let input = "#00FF19";
let rgb: RGB = input.parse().unwrap();
assert_eq!(rgb.to_string(), input);
```

# Color
## Try to parse a Color from a named color
```
# use std::str::FromStr as _;
# use twitchchat::color::*;
let input = "Blue Violet";
let color: Color = input.parse().unwrap();
assert_eq!(color.rgb, RGB(0x8A, 0x2B, 0xE2));
assert_eq!(color.kind, TwitchColor::BlueViolet);
```

# Conversions
```
# use twitchchat::color::*;
let input = "Blue Violet";
let color: Color = input.parse().unwrap();

// Color can be converted into an RGB
let rgb: RGB = color.into();

// RGB can be converted into a TwitchColor
let twitch_color: TwitchColor = rgb.into();

// TwitchColor can be converted into an RGB
let rgb: RGB = twitch_color.into();
```
*/

/// An error returned when trying to parse a string as an RGB triplet
#[non_exhaustive]
#[derive(Debug)]
pub enum ParseError {
    /// An invalid hex string for `RGB`
    InvalidHexString,
    /// Unknown color name
    UnknownColor,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidHexString => f.write_str("invalid hex string"),
            ParseError::UnknownColor => f.write_str("unknown color"),
        }
    }
}

impl std::error::Error for ParseError {}

impl std::str::FromStr for RGB {
    type Err = ParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        let input = match (input.chars().next(), input.len()) {
            (Some('#'), 7) => &input[1..],
            (.., 6) => input,
            _ => return Err(ParseError::InvalidHexString),
        };

        u32::from_str_radix(input, 16)
            .map(|s| {
                Self(
                    ((s >> 16) & 0xFF) as _,
                    ((s >> 8) & 0xFF) as _,
                    (s & 0xFF) as _,
                )
            })
            .map_err(|_| ParseError::InvalidHexString)
    }
}

/// A 24-bit RGB triplet
///
/// Default color is **white** `(0xFF,0xFF,0xFF)`
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct RGB(pub u8, pub u8, pub u8);

impl Default for RGB {
    fn default() -> Self {
        RGB(0xFF, 0xFF, 0xFF)
    }
}

impl std::fmt::Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b) = self;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}

impl RGB {
    /// The red field
    pub fn red(self) -> u8 {
        self.0
    }
    /// The green field
    pub fn green(self) -> u8 {
        self.1
    }
    /// The blue field
    pub fn blue(self) -> u8 {
        self.2
    }
}

/**
A twitch color paired with an RGB

Twitch has named colors. Users with `Turbo` enabled accounts can set a custom color.

Name | Color
--- | ---
Blue | `#0000FF`
BlueViolet | `#8A2BE2`
CadetBlue | `#5F9EA0`
Chocolate | `#D2691E`
Coral | `#FF7F50`
DodgerBlue | `#1E90FF`
Firebrick | `#B22222`
GoldenRod | `#DAA520`
Green | `#008000`
HotPink | `#FF69B4`
OrangeRed | `#FF4500`
Red | `#FF0000`
SeaGreen | `#2E8B57`
SpringGreen | `#00FF7F`
YellowGreen | `#ADFF2F`

These can be [parsed] from their **name** in
- `"PascalCase"`
- `"Title Case"`
- `"snake_case"`
- `"lower case"`

[parsed]: https://doc.rust-lang.org/std/str/trait.FromStr.html
*/
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Color {
    /// The name of the Twitch color
    pub kind: TwitchColor,
    /// The RGB triplet for this color
    pub rgb: RGB,
}

impl std::str::FromStr for Color {
    type Err = ParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        use TwitchColor::*;
        let find = |color| {
            let colors = twitch_colors();
            colors[colors.iter().position(|(d, _)| *d == color).unwrap()]
        };

        let mut s = input.replace(' ', "_");
        if !s.contains('_') {
            if let Some(pos) = s
                .chars()
                .skip(1)
                .position(|d| d.is_ascii_uppercase())
                .map(|d| d + 1)
            {
                s.insert(pos, '_');
            }
        }
        let s = s.to_ascii_lowercase();
        let (kind, rgb) = match s.as_str() {
            "blue" => find(Blue),
            "blue_violet" => find(BlueViolet),
            "cadet_blue" => find(CadetBlue),
            "chocolate" => find(Chocolate),
            "coral" => find(Coral),
            "dodger_blue" => find(DodgerBlue),
            "firebrick" => find(Firebrick),
            "golden_rod" => find(GoldenRod),
            "green" => find(Green),
            "hot_pink" => find(HotPink),
            "orange_red" => find(OrangeRed),
            "red" => find(Red),
            "sea_green" => find(SeaGreen),
            "spring_green" => find(SpringGreen),
            "yellow_green" => find(YellowGreen),
            _ => (Turbo, input.parse()?),
        };

        Ok(Self { kind, rgb })
    }
}

impl Default for Color {
    /// Defaults to having a kind of [Turbo] and RGB of #FFFFFF (white)
    ///
    /// [Turbo]: ./enum.TwitchColor.html#variant.Turbo
    fn default() -> Self {
        Self {
            kind: TwitchColor::Turbo,
            rgb: RGB::default(),
        }
    }
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
            _ => return f.write_str(&self.rgb.to_string()),
        };
        f.write_str(name)
    }
}

/// Named Twitch colors
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum TwitchColor {
    /// RGB (hex): `#0000FF`
    Blue,
    /// RGB (hex): `#8A2BE2`
    BlueViolet,
    /// RGB (hex): `#5F9EA0`
    CadetBlue,
    /// RGB (hex): `#D2691E`
    Chocolate,
    /// RGB (hex): `#FF7F50`
    Coral,
    /// RGB (hex): `#1E90FF`
    DodgerBlue,
    /// RGB (hex): `#B22222`
    Firebrick,
    /// RGB (hex): `#DAA520`
    GoldenRod,
    /// RGB (hex): `#008000`
    Green,
    /// RGB (hex): `#FF69B4`
    HotPink,
    /// RGB (hex): `#FF4500`
    OrangeRed,
    /// RGB (hex): `#FF0000`
    Red,
    /// RGB (hex): `#2E8B57`
    SeaGreen,
    /// RGB (hex): `#00FF7F`
    SpringGreen,
    /// RGB (hex): `#ADFF2F`
    YellowGreen,
    /// Turbo colors are custom user-selected colors
    Turbo,
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
            .map(|&(color, _)| color)
            .unwrap_or_else(|| TwitchColor::Turbo)
    }
}

impl From<TwitchColor> for RGB {
    fn from(color: TwitchColor) -> Self {
        twitch_colors()
            .iter()
            .find(|(c, _)| *c == color)
            .map(|&(_, rgb)| rgb)
            .unwrap_or_default()
    }
}

/// A utility method that returns an array of [TwitchColor]s mapped to its corresponding [RGB]
///
/// [TwitchColor]: ./enum.TwitchColor.html
/// [RGB]: ./struct.RGB.html
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
        use TwitchColor::*;
        let colors = vec![
            (Blue, vec!["Blue", "blue"]),
            (
                BlueViolet,
                vec!["BlueViolet", "Blue_Violet", "Blue Violet", "blue_violet"],
            ),
            (
                CadetBlue,
                vec!["CadetBlue", "Cadet_Blue", "Cadet Blue", "cadet_blue"],
            ),
            (Chocolate, vec!["Chocolate", "chocolate"]),
            (Coral, vec!["Coral", "coral"]),
            (
                DodgerBlue,
                vec!["DodgerBlue", "Dodger_Blue", "Dodger Blue", "dodger_blue"],
            ),
            (Firebrick, vec!["Firebrick", "firebrick"]),
            (
                GoldenRod,
                vec!["GoldenRod", "Golden_Rod", "Golden Rod", "golden_rod"],
            ),
            (Green, vec!["Green", "green"]),
            (HotPink, vec!["HotPink", "Hot_Pink", "Hot Pink", "hot_pink"]),
            (
                OrangeRed,
                vec!["OrangeRed", "Orange_Red", "Orange Red", "orange_red"],
            ),
            (Red, vec!["Red", "red"]),
            (
                SeaGreen,
                vec!["SeaGreen", "Sea_Green", "Sea Green", "sea_green"],
            ),
            (
                SpringGreen,
                vec![
                    "SpringGreen",
                    "Spring_Green",
                    "Spring Green",
                    "spring_green",
                ],
            ),
            (
                YellowGreen,
                vec![
                    "YellowGreen",
                    "Yellow_Green",
                    "Yellow Green",
                    "yellow_green",
                ],
            ),
        ];

        let twitch_colors = twitch_colors();
        for (tc, names) in colors {
            for name in names {
                let color: Color = name.parse().unwrap();
                assert_eq!(color.kind, tc);
                if let Some(rgb) = twitch_colors
                    .iter()
                    .find(|(c, _)| c == &color.kind)
                    .map(|(_, r)| r)
                {
                    assert_eq!(color.rgb, *rgb);
                }
            }
        }
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

    #[test]
    fn fields() {
        let rgb = RGB(0x27, 255, 82);
        assert_eq!(rgb.red(), 39);
        assert_eq!(rgb.green(), 255);
        assert_eq!(rgb.blue(), 82);
    }

    #[test]
    fn format_rgb() {
        let rgb = RGB(0x27, 255, 82);
        assert_eq!(rgb.to_string(), "#27FF52")
    }

    #[test]
    fn default_rgb() {
        let rgb = RGB::default();
        assert_eq!(rgb, RGB(0xFF, 0xFF, 0xFF))
    }
}
