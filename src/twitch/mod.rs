//! Common Twitch types

mod capability;
pub use capability::Capability;

mod userconfig;
pub use userconfig::{UserConfig, UserConfigBuilder, UserConfigError};

mod emotes;
pub use emotes::Emotes;

mod badge;
pub use badge::{Badge, BadgeInfo, BadgeKind};

pub mod color;
#[doc(inline)]
pub use color::Color;

pub(crate) fn parse_emotes(input: &str) -> Vec<Emotes> {
    Emotes::parse(input).collect()
}

pub(crate) fn parse_badges(input: &str) -> Vec<Badge<'_>> {
    input.split(',').filter_map(Badge::parse).collect()
}

#[allow(dead_code)]
pub(crate) fn parse_emotes_iter(input: &str) -> impl Iterator<Item = Emotes> + '_ {
    Emotes::parse(input)
}

pub(crate) fn parse_badges_iter(input: &str) -> impl Iterator<Item = Badge<'_>> + '_ {
    input.split(',').filter_map(Badge::parse)
}
