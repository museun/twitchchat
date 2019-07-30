mod badge;
mod color;
mod emotes;

// this is because a lot of things use super::* to get a HashMap
use std::collections::HashMap;

pub use badge::{Badge, BadgeInfo, BadgeKind};
pub use emotes::Emotes;

pub use color::{twitch_colors as colors, Color, TwitchColor, RGB};

/// An assortment of Twitch commands
pub mod commands;

mod capability;
pub use capability::Capability;

mod error;
pub use error::Error;

mod client;
pub use client::{Client, Event};

mod writer;
pub use writer::Writer;

/// Twitch channel types
mod channel;
pub use channel::{Channel, IntoChannel};

#[doc(hidden)]
pub mod userconfig;
pub use userconfig::UserConfig;
pub use userconfig::UserConfigBuilder;

mod local_user;
pub use local_user::LocalUser;

mod message;
pub use message::Message;

pub(crate) mod filter;
