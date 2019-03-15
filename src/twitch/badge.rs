/// BadgeKind are the `kind` of badges that are associated with messages.
///
/// Any unknown (e.g. custom badges/sub events, etc) are placed into the
/// `Unknown` variant
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Badge {
    /// The kind of Badge
    pub kind: BadgeKind,
    /// Any associated data with the badge
    ///
    /// May be the version, the number of bits, the number of months in a substreak
    pub data: String,
}

impl Badge {
    pub(crate) fn parse(input: &str) -> Option<Self> {
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
