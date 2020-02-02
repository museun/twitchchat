mod capability;
pub use capability::Capability;

mod userconfig;
pub use userconfig::{UserConfig, UserConfigBuilder, UserConfigError};

mod tags;
pub use tags::Tags;

mod emotes;
pub use emotes::Emotes;

mod badge;
pub use badge::{Badge, BadgeInfo, BadgeKind};

pub mod color;

mod channel;
pub use channel::{Channel, Error as ChannelError, IntoChannel};

pub(crate) fn parse_emotes<T>(input: &T) -> Vec<Emotes>
where
    T: crate::StringMarker,
{
    Emotes::parse(input.as_ref()).collect()
}

// TODO make this work with the conversion trait
pub(crate) fn parse_badges<'a>(input: &'a str) -> Vec<Badge<&'a str>> {
    input.split(',').filter_map(Badge::parse).collect()
}
