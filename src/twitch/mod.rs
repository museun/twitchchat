//! Common Twitch types

mod capability;
pub use capability::Capability;

// mod userconfig;
// pub use userconfig::{UserConfig, UserConfigBuilder, UserConfigError};

// mod tags;
// pub use tags::Tags;

mod emotes;
pub use emotes::Emotes;

mod badge;
pub use badge::{Badge, BadgeInfo, BadgeKind};

pub mod color;

mod channel;
pub use channel::{Channel, Error as ChannelError, IntoChannel};

// TODO provide an iterator as well as the Vec

pub(crate) fn parse_emotes(input: &str) -> Vec<Emotes> {
    Emotes::parse(input).collect()
}

pub(crate) fn parse_badges(input: &str) -> Vec<Badge<'_>> {
    input.split(',').filter_map(Badge::parse).collect()
}
