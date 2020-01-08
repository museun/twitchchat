/// The kind of the [badges] that are associated with messages.
///
/// Any unknonw (e.g. custom badges/sub events, etc) are placed into the [Unknown] variant.
///
/// [badges]: ./struct.Badge.html
/// [Unknown]: ./enum.BadgeKind.html#variant.Unknown
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum BadgeKind {
    /// Admin badge
    Admin,
    /// Bits badge
    Bits,
    /// Broadcaster badge
    Broadcaster,
    /// GlobalMod badge
    GlobalMod,
    /// Moderator badge
    Moderator,
    /// Subscriber badge
    Subscriber,
    /// Staff badge
    Staff,
    /// Turbo badge
    Turbo,
    /// Premium badge
    Premium,
    /// VIP badge
    VIP,
    /// Partner badge
    Partner,
    /// Unknown badge. Likely a custom badge
    Unknown(String),
}

/// Badges attached to a message
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Badge {
    /// The kind of the Badge
    pub kind: BadgeKind,
    /// Any associated data with the badge
    ///
    /// May be:
    /// - version
    /// - number of bits
    /// - number of months needed for sub badge
    /// - etc
    pub data: String,
}

impl Badge {
    /// Tries to parse a badge from this message part
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
            "premium" => Premium,
            "vip" => VIP,
            "partner" => Partner,
            badge => Unknown(badge.to_string()),
        };

        Self {
            kind,
            data: iter.next()?.to_string(),
        }
        .into()
    }
}

/// Metadata to the chat badges
pub type BadgeInfo = Badge;

// TODO tests
