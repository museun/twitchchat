/// The kind of the [badges] that are associated with messages.
///
/// Any unknown (e.g. custom badges/sub events, etc) are placed into the [Unknown] variant.
///
/// [badges]: Badge
/// [Unknown]: BadgeKind::Unknown
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum BadgeKind<'a> {
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
    Unknown(&'a str),
}

/// Badges attached to a message
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Badge<'a> {
    /// The kind of the Badge
    pub kind: BadgeKind<'a>,
    /// Any associated data with the badge
    ///
    /// May be:
    /// - version
    /// - number of bits
    /// - number of months needed for sub badge
    /// - etc
    pub data: &'a str,
}

impl<'a> Badge<'a> {
    /// Tries to parse a badge from this message part
    pub fn parse(input: &'a str) -> Option<Badge<'a>> {
        use BadgeKind::*;
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
            badge => Unknown(badge),
        };

        iter.next().map(|data| Badge { kind, data })
    }

    /// The `&str` representation of the [`BadgeKind`]
    ///
    /// In case of [`BadgeKind::Unknown`], this is the same value as `BadgeKind::Unknown(badge)`
    pub const fn kind_raw(&self) -> &'a str {
        use BadgeKind::*;
        match self.kind {
            Admin => "admin",
            Bits => "bits",
            Broadcaster => "broadcaster",
            GlobalMod => "global_mod",
            Moderator => "moderator",
            Subscriber => "subscriber",
            Staff => "staff",
            Turbo => "turbo",
            Premium => "premium",
            VIP => "vip",
            Partner => "partner",
            Unknown(s) => s,
        }
    }
}

/// Metadata to the chat badges
pub type BadgeInfo<'a> = Badge<'a>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_badges() {
        // ("input", expected value)
        const BADGE_KINDS: &[(&str, BadgeKind<'_>)] = &[
            ("admin", BadgeKind::Admin),
            ("bits", BadgeKind::Bits),
            ("broadcaster", BadgeKind::Broadcaster),
            ("global_mod", BadgeKind::GlobalMod),
            ("moderator", BadgeKind::Moderator),
            ("subscriber", BadgeKind::Subscriber),
            ("staff", BadgeKind::Staff),
            ("turbo", BadgeKind::Turbo),
            ("premium", BadgeKind::Premium),
            ("vip", BadgeKind::VIP),
            ("partner", BadgeKind::Partner),
            ("unknown", BadgeKind::Unknown("unknown")),
        ];

        for (raw, kind) in BADGE_KINDS {
            let badge_str = format!("{}/0", raw);
            let badge = Badge::parse(&badge_str).expect("Malformed badge test");

            assert_eq!(badge.kind, *kind);
            assert_eq!(badge.kind_raw(), *raw);
            assert_eq!(badge.data, "0");
        }
    }

    #[test]
    fn parse_unknown() {
        let badge_str = "this_badge_does_not_exist/0";
        let badge = Badge::parse(badge_str).unwrap();
        assert_eq!(
            badge,
            Badge {
                kind: BadgeKind::Unknown("this_badge_does_not_exist"),
                data: "0"
            }
        );

        assert_eq!(badge.kind_raw(), "this_badge_does_not_exist")
    }

    #[test]
    fn parse_invalid() {
        let badge_str = "this_badge_is_invalid";
        let badge = Badge::parse(badge_str);
        assert_eq!(badge, None)
    }
}
