#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BadgeKind {
    Admin,
    Bits,
    Broadcaster,
    GlobalMod,
    Moderator,
    Subscriber,
    Staff,
    Turbo,
    Unknown(String),
}

/// Badges attached to a message
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Badge {
    pub kind: BadgeKind,
    pub data: String,
}

impl Badge {
    pub fn parse(input: &str) -> Option<Self> {
        use BadgeKind::*;
        let input = input.to_ascii_lowercase();
        let mut iter = input.split('/');
        let kind = match iter.next()? {
            "admin" => Admin,
            "bits" => Bits,
            "broadcaster" => Broadcaster,
            "global_mod" => GlobalMod,
            "moderator" => Moderator,
            "subscriber" => Subscriber,
            "staff" => Staff,
            "turbo" => Turbo,
            badge => Unknown(badge.to_string()),
        };
        Some(Badge {
            kind,
            data: iter.next()?.to_string(),
        })
    }
}
