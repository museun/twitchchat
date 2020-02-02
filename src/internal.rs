use crate::Conversion;
use std::{borrow::Borrow, fmt::Debug, hash::Hash};

/// Marker trait for the 'storage' abstraction of the types
pub trait StringMarker
where
    Self: Hash + Debug + Clone,
    Self: Eq + PartialEq + AsRef<str> + Borrow<str>,
    Self: private::string_marker::Sealed,
{
}

impl StringMarker for String {}
impl<'a> StringMarker for &'a str {}

impl<'a> Conversion<'a> for bool {
    type Borrowed = bool;
    type Owned = bool;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        *self
    }
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl<'a> Conversion<'a> for usize {
    type Borrowed = usize;
    type Owned = usize;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        *self
    }
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl<'a> Conversion<'a> for &'a str {
    type Borrowed = &'a str;
    type Owned = String;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        self
    }
    fn as_owned(&self) -> Self::Owned {
        self.to_string()
    }
}

impl<'a> Conversion<'a> for String {
    type Borrowed = &'a str;
    type Owned = String;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        self.as_str()
    }
    fn as_owned(&self) -> Self::Owned {
        self.clone()
    }
}

impl<'a, T> Conversion<'a> for Option<T>
where
    T: Conversion<'a>,
{
    type Borrowed = Option<T::Borrowed>;
    type Owned = Option<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        self.as_ref().map(|s| s.as_borrowed())
    }
    fn as_owned(&self) -> Self::Owned {
        match self {
            Some(item) => Some(item.as_owned()),
            None => None,
        }
    }
}

impl<'a, T> Conversion<'a> for Vec<T>
where
    T: Conversion<'a>,
{
    type Borrowed = Vec<T::Borrowed>;
    type Owned = Vec<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        self.iter().map(|s| s.as_borrowed()).collect()
    }
    fn as_owned(&self) -> Self::Owned {
        self.iter().map(|s| s.as_owned()).collect()
    }
}

impl<'a, T> Conversion<'a> for crate::Badge<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::Badge<T::Borrowed>;
    type Owned = crate::Badge<T::Owned>;

    fn as_borrowed(&'a self) -> Self::Borrowed {
        crate::Badge {
            kind: self.kind.as_borrowed(),
            data: self.data.as_borrowed(),
        }
    }
    fn as_owned(&self) -> Self::Owned {
        crate::Badge {
            kind: self.kind.as_owned(),
            data: self.data.as_owned(),
        }
    }
}

impl<'a, T> Conversion<'a> for crate::BadgeKind<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::BadgeKind<T::Borrowed>;
    type Owned = crate::BadgeKind<T::Owned>;

    fn as_borrowed(&'a self) -> Self::Borrowed {
        use crate::BadgeKind::*;
        match self {
            Admin => Admin,
            Bits => Bits,
            Broadcaster => Broadcaster,
            GlobalMod => GlobalMod,
            Moderator => Moderator,
            Subscriber => Subscriber,
            Staff => Staff,
            Turbo => Turbo,
            Premium => Premium,
            VIP => VIP,
            Partner => Partner,
            Unknown(inner) => Unknown(inner.as_borrowed()),
        }
    }
    fn as_owned(&self) -> Self::Owned {
        use crate::BadgeKind::*;
        match self {
            Admin => Admin,
            Bits => Bits,
            Broadcaster => Broadcaster,
            GlobalMod => GlobalMod,
            Moderator => Moderator,
            Subscriber => Subscriber,
            Staff => Staff,
            Turbo => Turbo,
            Premium => Premium,
            VIP => VIP,
            Partner => Partner,
            Unknown(inner) => Unknown(inner.as_owned()),
        }
    }
}

impl<'a> Conversion<'a> for crate::color::Color {
    type Borrowed = crate::color::Color;
    type Owned = crate::color::Color;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        *self
    }
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl<'a, T> Conversion<'a> for crate::Tags<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::Tags<T::Borrowed>;
    type Owned = crate::Tags<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        crate::Tags(
            self.0
                .iter()
                .map(|(k, v)| (k.as_borrowed(), v.as_borrowed()))
                .collect(),
        )
    }
    fn as_owned(&self) -> Self::Owned {
        crate::Tags(
            self.0
                .iter()
                .map(|(k, v)| (k.as_owned(), v.as_owned()))
                .collect(),
        )
    }
}

impl<'a, T> Conversion<'a> for crate::messages::NamesKind<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::messages::NamesKind<T::Borrowed>;
    type Owned = crate::messages::NamesKind<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        match self {
            crate::messages::NamesKind::Start { users } => crate::messages::NamesKind::Start {
                users: users.as_borrowed(),
            },
            crate::messages::NamesKind::End => crate::messages::NamesKind::End,
        }
    }
    fn as_owned(&self) -> Self::Owned {
        match self {
            crate::messages::NamesKind::Start { users } => crate::messages::NamesKind::Start {
                users: users.as_owned(),
            },
            crate::messages::NamesKind::End => crate::messages::NamesKind::End,
        }
    }
}

impl<'a, T> Conversion<'a> for crate::messages::HostTargetKind<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::messages::HostTargetKind<T::Borrowed>;
    type Owned = crate::messages::HostTargetKind<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        match self {
            crate::messages::HostTargetKind::Start { target } => {
                crate::messages::HostTargetKind::Start {
                    target: target.as_borrowed(),
                }
            }
            crate::messages::HostTargetKind::End => crate::messages::HostTargetKind::End,
        }
    }
    fn as_owned(&self) -> Self::Owned {
        match self {
            crate::messages::HostTargetKind::Start { target } => {
                crate::messages::HostTargetKind::Start {
                    target: target.as_owned(),
                }
            }
            crate::messages::HostTargetKind::End => crate::messages::HostTargetKind::End,
        }
    }
}

impl<'a> Conversion<'a> for crate::messages::ModeStatus {
    type Borrowed = crate::messages::ModeStatus;
    type Owned = crate::messages::ModeStatus;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        *self
    }
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}

impl<'a, T> Conversion<'a> for crate::messages::NoticeType<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::messages::NoticeType<T::Borrowed>;
    type Owned = crate::messages::NoticeType<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        use crate::messages::NoticeType::*;
        match self {
            Sub => Sub,
            Resub => Resub,
            SubGift => SubGift,
            AnonSubGift => AnonSubGift,
            SubMysteryGift => SubMysteryGift,
            GiftPaidUpgrade => GiftPaidUpgrade,
            RewardGift => RewardGift,
            AnonGiftPaidUpgrade => AnonGiftPaidUpgrade,
            Raid => Raid,
            Unraid => Unraid,
            Ritual => Ritual,
            BitsBadgeTier => BitsBadgeTier,
            Unknown(data) => Unknown(data.as_borrowed()),
        }
    }
    fn as_owned(&self) -> Self::Owned {
        use crate::messages::NoticeType::*;
        match self {
            Sub => Sub,
            Resub => Resub,
            SubGift => SubGift,
            AnonSubGift => AnonSubGift,
            SubMysteryGift => SubMysteryGift,
            GiftPaidUpgrade => GiftPaidUpgrade,
            RewardGift => RewardGift,
            AnonGiftPaidUpgrade => AnonGiftPaidUpgrade,
            Raid => Raid,
            Unraid => Unraid,
            Ritual => Ritual,
            BitsBadgeTier => BitsBadgeTier,
            Unknown(data) => Unknown(data.as_owned()),
        }
    }
}

impl<'a, T> Conversion<'a> for crate::messages::MessageId<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::messages::MessageId<T::Borrowed>;
    type Owned = crate::messages::MessageId<T::Owned>;
    fn as_borrowed(&'a self) -> Self::Borrowed {
        use crate::messages::MessageId::*;
        match self {
            AlreadyBanned => AlreadyBanned,
            AlreadyEmoteOnlyOff => AlreadyEmoteOnlyOff,
            AlreadyEmoteOnlyOn => AlreadyEmoteOnlyOn,
            AlreadyR9kOff => AlreadyR9kOff,
            AlreadyR9kOn => AlreadyR9kOn,
            AlreadySubsOff => AlreadySubsOff,
            AlreadySubsOn => AlreadySubsOn,
            BadBanAdmin => BadBanAdmin,
            BadBanAnon => BadBanAnon,
            BadBanBroadcaster => BadBanBroadcaster,
            BadBanGlobalMod => BadBanGlobalMod,
            BadBanMod => BadBanMod,
            BadBanSelf => BadBanSelf,
            BadBanStaff => BadBanStaff,
            BadCommercialError => BadCommercialError,
            BadDeleteMessageBroadcaster => BadDeleteMessageBroadcaster,
            BadDeleteMessageMod => BadDeleteMessageMod,
            BadHostError => BadHostError,
            BadHostHosting => BadHostHosting,
            BadHostRateExceeded => BadHostRateExceeded,
            BadHostRejected => BadHostRejected,
            BadHostSelf => BadHostSelf,
            BadMarkerClient => BadMarkerClient,
            BadModBanned => BadModBanned,
            BadModMod => BadModMod,
            BadSlowDuration => BadSlowDuration,
            BadTimeoutAdmin => BadTimeoutAdmin,
            BadTimeoutAnon => BadTimeoutAnon,
            BadTimeoutBroadcaster => BadTimeoutBroadcaster,
            BadTimeoutDuration => BadTimeoutDuration,
            BadTimeoutGlobalMod => BadTimeoutGlobalMod,
            BadTimeoutMod => BadTimeoutMod,
            BadTimeoutSelf => BadTimeoutSelf,
            BadTimeoutStaff => BadTimeoutStaff,
            BadUnbanNoBan => BadUnbanNoBan,
            BadUnhostError => BadUnhostError,
            BadUnmodMod => BadUnmodMod,
            BanSuccess => BanSuccess,
            CmdsAvailable => CmdsAvailable,
            ColorChanged => ColorChanged,
            CommercialSuccess => CommercialSuccess,
            DeleteMessageSuccess => DeleteMessageSuccess,
            EmoteOnlyOff => EmoteOnlyOff,
            EmoteOnlyOn => EmoteOnlyOn,
            FollowersOff => FollowersOff,
            FollowersOn => FollowersOn,
            FollowersOnZero => FollowersOnZero,
            HostOff => HostOff,
            HostOn => HostOn,
            HostSuccess => HostSuccess,
            HostSuccessViewers => HostSuccessViewers,
            HostTargetWentOffline => HostTargetWentOffline,
            HostsRemaining => HostsRemaining,
            InvalidUser => InvalidUser,
            ModSuccess => ModSuccess,
            MsgBanned => MsgBanned,
            MsgBadCharacters => MsgBadCharacters,
            MsgChannelBlocked => MsgChannelBlocked,
            MsgChannelSuspended => MsgChannelSuspended,
            MsgDuplicate => MsgDuplicate,
            MsgEmoteonly => MsgEmoteonly,
            MsgFacebook => MsgFacebook,
            MsgFollowersonly => MsgFollowersonly,
            MsgFollowersonlyFollowed => MsgFollowersonlyFollowed,
            MsgFollowersonlyZero => MsgFollowersonlyZero,
            MsgR9k => MsgR9k,
            MsgRatelimit => MsgRatelimit,
            MsgRejected => MsgRejected,
            MsgRejectedMandatory => MsgRejectedMandatory,
            MsgRoomNotFound => MsgRoomNotFound,
            MsgSlowmode => MsgSlowmode,
            MsgSubsonly => MsgSubsonly,
            MsgSuspended => MsgSuspended,
            MsgTimedout => MsgTimedout,
            MsgVerifiedEmail => MsgVerifiedEmail,
            NoHelp => NoHelp,
            NoMods => NoMods,
            NotHosting => NotHosting,
            NoPermission => NoPermission,
            R9kOff => R9kOff,
            R9kOn => R9kOn,
            RaidErrorAlreadyRaiding => RaidErrorAlreadyRaiding,
            RaidErrorForbidden => RaidErrorForbidden,
            RaidErrorSelf => RaidErrorSelf,
            RaidErrorTooManyViewers => RaidErrorTooManyViewers,
            RaidErrorUnexpected => RaidErrorUnexpected,
            RaidNoticeMature => RaidNoticeMature,
            RaidNoticeRestrictedChat => RaidNoticeRestrictedChat,
            RoomMods => RoomMods,
            SlowOff => SlowOff,
            SlowOn => SlowOn,
            SubsOff => SubsOff,
            SubsOn => SubsOn,
            TimeoutNoTimeout => TimeoutNoTimeout,
            TimeoutSuccess => TimeoutSuccess,
            TosBan => TosBan,
            TurboOnlyColor => TurboOnlyColor,
            UnbanSuccess => UnbanSuccess,
            UnmodSuccess => UnmodSuccess,
            UnraidErrorNoActiveRaid => UnraidErrorNoActiveRaid,
            UnraidErrorUnexpected => UnraidErrorUnexpected,
            UnraidSuccess => UnraidSuccess,
            UnrecognizedCmd => UnrecognizedCmd,
            UnsupportedChatroomsCmd => UnsupportedChatroomsCmd,
            UntimeoutBanned => UntimeoutBanned,
            UntimeoutSuccess => UntimeoutSuccess,
            UsageBan => UsageBan,
            UsageClear => UsageClear,
            UsageColor => UsageColor,
            UsageCommercial => UsageCommercial,
            UsageDisconnect => UsageDisconnect,
            UsageEmoteOnlyOff => UsageEmoteOnlyOff,
            UsageEmoteOnlyOn => UsageEmoteOnlyOn,
            UsageFollowersOff => UsageFollowersOff,
            UsageFollowersOn => UsageFollowersOn,
            UsageHelp => UsageHelp,
            UsageHost => UsageHost,
            UsageMarker => UsageMarker,
            UsageMe => UsageMe,
            UsageMod => UsageMod,
            UsageMods => UsageMods,
            UsageR9kOff => UsageR9kOff,
            UsageR9kOn => UsageR9kOn,
            UsageRaid => UsageRaid,
            UsageSlowOff => UsageSlowOff,
            UsageSlowOn => UsageSlowOn,
            UsageSubsOff => UsageSubsOff,
            UsageSubsOn => UsageSubsOn,
            UsageTimeout => UsageTimeout,
            UsageUnban => UsageUnban,
            UsageUnhost => UsageUnhost,
            UsageUnmod => UsageUnmod,
            UsageUnraid => UsageUnraid,
            UsageUntimeout => UsageUntimeout,
            WhisperBanned => WhisperBanned,
            WhisperBannedRecipient => WhisperBannedRecipient,
            WhisperInvalidArgs => WhisperInvalidArgs,
            WhisperInvalidLogin => WhisperInvalidLogin,
            WhisperInvalidSelf => WhisperInvalidSelf,
            WhisperLimitPerMin => WhisperLimitPerMin,
            WhisperLimitPerSec => WhisperLimitPerSec,
            WhisperRestricted => WhisperRestricted,
            WhisperRestrictedRecipient => WhisperRestrictedRecipient,
            Unknown(data) => Unknown(data.as_borrowed()),
        }
    }
    fn as_owned(&self) -> Self::Owned {
        use crate::messages::MessageId::*;
        match self {
            AlreadyBanned => AlreadyBanned,
            AlreadyEmoteOnlyOff => AlreadyEmoteOnlyOff,
            AlreadyEmoteOnlyOn => AlreadyEmoteOnlyOn,
            AlreadyR9kOff => AlreadyR9kOff,
            AlreadyR9kOn => AlreadyR9kOn,
            AlreadySubsOff => AlreadySubsOff,
            AlreadySubsOn => AlreadySubsOn,
            BadBanAdmin => BadBanAdmin,
            BadBanAnon => BadBanAnon,
            BadBanBroadcaster => BadBanBroadcaster,
            BadBanGlobalMod => BadBanGlobalMod,
            BadBanMod => BadBanMod,
            BadBanSelf => BadBanSelf,
            BadBanStaff => BadBanStaff,
            BadCommercialError => BadCommercialError,
            BadDeleteMessageBroadcaster => BadDeleteMessageBroadcaster,
            BadDeleteMessageMod => BadDeleteMessageMod,
            BadHostError => BadHostError,
            BadHostHosting => BadHostHosting,
            BadHostRateExceeded => BadHostRateExceeded,
            BadHostRejected => BadHostRejected,
            BadHostSelf => BadHostSelf,
            BadMarkerClient => BadMarkerClient,
            BadModBanned => BadModBanned,
            BadModMod => BadModMod,
            BadSlowDuration => BadSlowDuration,
            BadTimeoutAdmin => BadTimeoutAdmin,
            BadTimeoutAnon => BadTimeoutAnon,
            BadTimeoutBroadcaster => BadTimeoutBroadcaster,
            BadTimeoutDuration => BadTimeoutDuration,
            BadTimeoutGlobalMod => BadTimeoutGlobalMod,
            BadTimeoutMod => BadTimeoutMod,
            BadTimeoutSelf => BadTimeoutSelf,
            BadTimeoutStaff => BadTimeoutStaff,
            BadUnbanNoBan => BadUnbanNoBan,
            BadUnhostError => BadUnhostError,
            BadUnmodMod => BadUnmodMod,
            BanSuccess => BanSuccess,
            CmdsAvailable => CmdsAvailable,
            ColorChanged => ColorChanged,
            CommercialSuccess => CommercialSuccess,
            DeleteMessageSuccess => DeleteMessageSuccess,
            EmoteOnlyOff => EmoteOnlyOff,
            EmoteOnlyOn => EmoteOnlyOn,
            FollowersOff => FollowersOff,
            FollowersOn => FollowersOn,
            FollowersOnZero => FollowersOnZero,
            HostOff => HostOff,
            HostOn => HostOn,
            HostSuccess => HostSuccess,
            HostSuccessViewers => HostSuccessViewers,
            HostTargetWentOffline => HostTargetWentOffline,
            HostsRemaining => HostsRemaining,
            InvalidUser => InvalidUser,
            ModSuccess => ModSuccess,
            MsgBanned => MsgBanned,
            MsgBadCharacters => MsgBadCharacters,
            MsgChannelBlocked => MsgChannelBlocked,
            MsgChannelSuspended => MsgChannelSuspended,
            MsgDuplicate => MsgDuplicate,
            MsgEmoteonly => MsgEmoteonly,
            MsgFacebook => MsgFacebook,
            MsgFollowersonly => MsgFollowersonly,
            MsgFollowersonlyFollowed => MsgFollowersonlyFollowed,
            MsgFollowersonlyZero => MsgFollowersonlyZero,
            MsgR9k => MsgR9k,
            MsgRatelimit => MsgRatelimit,
            MsgRejected => MsgRejected,
            MsgRejectedMandatory => MsgRejectedMandatory,
            MsgRoomNotFound => MsgRoomNotFound,
            MsgSlowmode => MsgSlowmode,
            MsgSubsonly => MsgSubsonly,
            MsgSuspended => MsgSuspended,
            MsgTimedout => MsgTimedout,
            MsgVerifiedEmail => MsgVerifiedEmail,
            NoHelp => NoHelp,
            NoMods => NoMods,
            NotHosting => NotHosting,
            NoPermission => NoPermission,
            R9kOff => R9kOff,
            R9kOn => R9kOn,
            RaidErrorAlreadyRaiding => RaidErrorAlreadyRaiding,
            RaidErrorForbidden => RaidErrorForbidden,
            RaidErrorSelf => RaidErrorSelf,
            RaidErrorTooManyViewers => RaidErrorTooManyViewers,
            RaidErrorUnexpected => RaidErrorUnexpected,
            RaidNoticeMature => RaidNoticeMature,
            RaidNoticeRestrictedChat => RaidNoticeRestrictedChat,
            RoomMods => RoomMods,
            SlowOff => SlowOff,
            SlowOn => SlowOn,
            SubsOff => SubsOff,
            SubsOn => SubsOn,
            TimeoutNoTimeout => TimeoutNoTimeout,
            TimeoutSuccess => TimeoutSuccess,
            TosBan => TosBan,
            TurboOnlyColor => TurboOnlyColor,
            UnbanSuccess => UnbanSuccess,
            UnmodSuccess => UnmodSuccess,
            UnraidErrorNoActiveRaid => UnraidErrorNoActiveRaid,
            UnraidErrorUnexpected => UnraidErrorUnexpected,
            UnraidSuccess => UnraidSuccess,
            UnrecognizedCmd => UnrecognizedCmd,
            UnsupportedChatroomsCmd => UnsupportedChatroomsCmd,
            UntimeoutBanned => UntimeoutBanned,
            UntimeoutSuccess => UntimeoutSuccess,
            UsageBan => UsageBan,
            UsageClear => UsageClear,
            UsageColor => UsageColor,
            UsageCommercial => UsageCommercial,
            UsageDisconnect => UsageDisconnect,
            UsageEmoteOnlyOff => UsageEmoteOnlyOff,
            UsageEmoteOnlyOn => UsageEmoteOnlyOn,
            UsageFollowersOff => UsageFollowersOff,
            UsageFollowersOn => UsageFollowersOn,
            UsageHelp => UsageHelp,
            UsageHost => UsageHost,
            UsageMarker => UsageMarker,
            UsageMe => UsageMe,
            UsageMod => UsageMod,
            UsageMods => UsageMods,
            UsageR9kOff => UsageR9kOff,
            UsageR9kOn => UsageR9kOn,
            UsageRaid => UsageRaid,
            UsageSlowOff => UsageSlowOff,
            UsageSlowOn => UsageSlowOn,
            UsageSubsOff => UsageSubsOff,
            UsageSubsOn => UsageSubsOn,
            UsageTimeout => UsageTimeout,
            UsageUnban => UsageUnban,
            UsageUnhost => UsageUnhost,
            UsageUnmod => UsageUnmod,
            UsageUnraid => UsageUnraid,
            UsageUntimeout => UsageUntimeout,
            WhisperBanned => WhisperBanned,
            WhisperBannedRecipient => WhisperBannedRecipient,
            WhisperInvalidArgs => WhisperInvalidArgs,
            WhisperInvalidLogin => WhisperInvalidLogin,
            WhisperInvalidSelf => WhisperInvalidSelf,
            WhisperLimitPerMin => WhisperLimitPerMin,
            WhisperLimitPerSec => WhisperLimitPerSec,
            WhisperRestricted => WhisperRestricted,
            WhisperRestrictedRecipient => WhisperRestrictedRecipient,
            Unknown(data) => Unknown(data.as_owned()),
        }
    }
}

impl<'a, T> Conversion<'a> for crate::decode::Message<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::decode::Message<T::Borrowed>;
    type Owned = crate::decode::Message<T::Owned>;

    fn as_borrowed(&'a self) -> Self::Borrowed {
        crate::decode::Message {
            raw: self.raw.as_borrowed(),
            tags: self.tags.as_borrowed(),
            prefix: self.prefix.as_borrowed(),
            command: self.command.as_borrowed(),
            args: self.args.as_borrowed(),
            data: self.data.as_borrowed(),
        }
    }

    fn as_owned(&self) -> Self::Owned {
        crate::decode::Message {
            raw: self.raw.as_owned(),
            tags: self.tags.as_owned(),
            prefix: self.prefix.as_owned(),
            command: self.command.as_owned(),
            args: self.args.as_owned(),
            data: self.data.as_owned(),
        }
    }
}

impl<'a, T> Conversion<'a> for crate::decode::Prefix<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Borrowed = crate::decode::Prefix<T::Borrowed>;
    type Owned = crate::decode::Prefix<T::Owned>;

    fn as_borrowed(&'a self) -> Self::Borrowed {
        match self {
            crate::decode::Prefix::User { nick } => crate::decode::Prefix::User {
                nick: nick.as_borrowed(),
            },
            crate::decode::Prefix::Server { host } => crate::decode::Prefix::Server {
                host: host.as_borrowed(),
            },
        }
    }

    fn as_owned(&self) -> Self::Owned {
        match self {
            crate::decode::Prefix::User { nick } => crate::decode::Prefix::User {
                nick: nick.as_owned(),
            },
            crate::decode::Prefix::Server { host } => crate::decode::Prefix::Server {
                host: host.as_owned(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bool_conversion() {
        let ok = false;
        assert_eq!(ok.as_borrowed(), ok);
        assert_eq!(ok.as_owned(), ok);
    }

    #[test]
    fn usize_conversion() {
        let ok = 42_usize;
        assert_eq!(ok.as_borrowed(), ok);
        assert_eq!(ok.as_owned(), ok);
    }

    #[test]
    fn string_conversion() {
        let owned = String::from("foobar");
        let borrowed = "foobar";

        assert_eq!(owned.as_borrowed(), borrowed);
        assert_eq!(borrowed.as_owned(), owned);
    }

    #[test]
    fn badge_conversion() {
        let input_owned = crate::Badge {
            kind: crate::BadgeKind::Unknown("foobar".to_string()),
            data: "asdf".to_string(),
        };

        let input_borrowed = crate::Badge {
            kind: crate::BadgeKind::Unknown("foobar"),
            data: "asdf",
        };

        assert_eq!(input_owned.as_borrowed(), input_borrowed);
        assert_eq!(input_borrowed.as_owned(), input_owned);

        assert_eq!(input_owned.as_borrowed().as_owned(), input_owned);
        assert_eq!(input_borrowed.as_owned().as_borrowed(), input_borrowed);
    }

    #[test]
    #[allow(unused_qualifications)]
    fn option_conversion() {
        let owned = Some(String::from("asdf"));

        assert_eq!(owned.as_borrowed(), Some("asdf"));
        assert_eq!(Some("asdf").as_owned(), owned);

        assert_eq!(Option::<String>::None.as_borrowed(), None);
        assert_eq!(Option::<&'static str>::None.as_owned(), None);
    }

    #[test]
    fn vec_conversion() {
        let list = (b'a'..=b'z')
            .map(|s| (s as char).to_string())
            .map(Some)
            .collect::<Vec<_>>();

        let ref_ = list.iter().map(|s| s.as_deref()).collect::<Vec<_>>();

        assert_eq!(list.as_borrowed(), ref_);
        assert_eq!(ref_.as_owned(), list);
    }
}

pub(crate) mod private {
    pub(crate) mod string_marker {
        pub trait Sealed {}
        impl<T> Sealed for T where T: crate::internal::StringMarker {}
    }

    pub(crate) mod parse_marker {
        pub trait Sealed<E> {}
        impl<T, E> Sealed<E> for T
        where
            T: crate::Parse<E>,
            E: Sized,
        {
        }
    }
}
