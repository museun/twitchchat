use super::*;

/// Announces Twitch-specific events to the channel (e.g., a user's subscription notification).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserNotice<'t> {
    /// Tags attached to this message
    pub tags: Tags<'t>,
    /// Channel this event is happening on
    pub channel: Cow<'t, str>,
    /// Optional message attached to the event
    pub message: Option<Cow<'t, str>>,
}

impl<'t> UserNotice<'t> {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber
    ///    
    pub fn badge_info(&'t self) -> Vec<crate::BadgeInfo<'t>> {
        self.tags
            .get_ref("badge-info")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// Badges attached to this message
    ///    
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get_ref("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// The user's color, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// The user's display name, if set
    pub fn display_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(&s))
            .unwrap_or_default()
    }

    /// A unique id (UUID) attached to this message
    ///
    /// (this is used for localization)
    pub fn id(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("id")
    }

    /// The name of the user who sent this notice
    pub fn login(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("login")
    }

    /// Whether this user is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags.get_as_bool("mod")
    }

    /// The kind of notice this message is
    pub fn msg_id(&'t self) -> Option<NoticeType<'t>> {
        let kind = self.tags.get_ref("msg-id")?;
        match kind.as_ref() {
            "sub" => NoticeType::Sub,
            "resub" => NoticeType::Resub,
            "subgift" => NoticeType::SubGift,
            "anonsubgift" => NoticeType::AnonSubGift,
            "submysterygift" => NoticeType::SubMysteryGift,
            "giftpaidupgrade" => NoticeType::GiftPaidUpgrade,
            "rewardgift" => NoticeType::RewardGift,
            "anongiftpaidupgrade" => NoticeType::AnonGiftPaidUpgrade,
            "raid" => NoticeType::Raid,
            "unraid" => NoticeType::Unraid,
            "ritual" => NoticeType::Ritual,
            "bitsbadgetier" => NoticeType::BitsBadgeTier,
            _ => NoticeType::Unknown(kind.reborrow()),
        }
        .into()
    }

    /// The id of the room for this notice
    pub fn room_id(&self) -> Option<u64> {
        self.tags.get_parsed("room-id")
    }

    /// THe timestamp which twitch received this message
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags.get_parsed("tmi-sent-ts")
    }

    /// User id of the user who sent this notice
    pub fn user_id(&self) -> Option<u64> {
        self.tags.get_parsed("user-id")
    }

    /// The message printed in chat along with this notice
    pub fn system_msg(&self) -> Option<String> {
        self.tags
            .get("system-msg")?
            .as_ref()
            .replace("\\s", " ")
            .replace("\\r", "\r")
            .replace("\\n", "\n")
            .replace("\\\\", "\\")
            .replace("\\:", ":")
            .into()
    }

    /// (Sent only on sub, resub) The total number of months the user has
    /// subscribed. This is the same as msg-param-months but sent for different
    /// types of user notices.
    pub fn msg_param_cumulative_months(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-cumulative-months")
    }

    /// (Sent only on raid) The display name of the source user raiding this
    /// channel.
    pub fn msg_param_display_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-displayName")
    }

    /// (Sent on only raid) The name of the source user raiding this channel.
    pub fn msg_param_login(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-login")
    }

    /// (Sent only on subgift, anonsubgift) The total number of months the user
    /// has subscribed. This is the same as msg-param-cumulative-months but sent
    /// for different types of user notices.
    pub fn msg_param_months(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-months")
    }

    /// (Sent only on anongiftpaidupgrade, giftpaidupgrade) The number of gifts
    /// the gifter has given during the promo indicated by msg-param-promo-name.
    pub fn msg_param_promo_gift_total(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-promo-gift-total")
    }

    /// (Sent only on anongiftpaidupgrade, giftpaidupgrade) The subscriptions
    /// promo, if any, that is ongoing; e.g. Subtember 2018.
    pub fn msg_param_promo_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-promo-name")
    }

    /// (Sent only on subgift, anonsubgift) The display name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_display_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-recipient-display-name")
    }

    /// (Sent only on subgift, anonsubgift) The user ID of the subscription gift
    /// recipient.
    pub fn msg_param_recipient_id(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-recipient-id")
    }

    /// (Sent only on subgift, anonsubgift) The user name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_user_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-recipient-user-name")
    }

    /// (Sent only on giftpaidupgrade) The login of the user who gifted the
    /// subscription.
    pub fn msg_param_sender_login(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-sender-login")
    }

    /// (Sent only on giftpaidupgrade) The display name of the user who gifted
    /// the subscription.
    pub fn msg_param_sender_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-sender-name")
    }

    /// (Sent only on sub, resub) Boolean indicating whether users want their
    /// streaks to be shared.
    pub fn msg_param_should_share_streak(&self) -> Option<bool> {
        self.tags.get_parsed("msg-param-should-share-streak")
    }

    /// (Sent only on sub, resub) The number of consecutive months the user has
    /// subscribed. This is 0 if msg-param-should-share-streak is 0.
    pub fn msg_param_streak_months(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-streak-months")
    }

    /// (Sent only on sub, resub, subgift, anonsubgift) The type of subscription
    /// plan being used. Valid values: Prime, 1000, 2000, 3000. 1000, 2000, and
    /// 3000 refer to the first, second, and third levels of paid subscriptions,
    /// respectively (currently $4.99, $9.99, and $24.99).
    pub fn msg_param_sub_plan(&self) -> Option<SubPlan> {
        self.tags.get("msg-param-sub-plan").and_then(|s| {
            match s.as_ref() {
                "Prime" => SubPlan::Prime,
                "Tier1" => SubPlan::Tier1,
                "Tier2" => SubPlan::Tier2,
                "Tier3" => SubPlan::Tier3,
                _ => return None, // TODO warn on this?
            }
            .into()
        })
    }

    /// (Sent only on sub, resub, subgift, anonsubgift) The display name of the
    /// subscription plan. This may be a default name or one created by the
    /// channel owner.
    pub fn msg_param_sub_plan_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-sub-plan-name")
    }

    /// (Sent only on raid) The number of viewers watching the source channel
    /// raiding this channel.
    pub fn msg_param_viewer_count(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-viewerCount")
    }

    /// (Sent only on ritual) The name of the ritual this notice is for. Valid
    /// value: new_chatter.
    pub fn msg_param_ritual_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("msg-param-ritual-name")
    }

    /// (Sent only on bitsbadgetier) The tier of the bits badge the user just
    /// earned; e.g. 100, 1000, 10000.
    pub fn msg_param_threshold(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-threshold")
    }
}

/// A paid subscription ot the channel
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubPlan {
    /// A `Prime` subscription
    Prime,
    /// A Tier-1 subscription (currently $4.99)
    Tier1,
    /// A Tier-2 subscription (currently $9.99)
    Tier2,
    /// A Tier-3 subscription (currently $24.99)
    Tier3,
}

/// The kind of notice it was, retrieved via [`UserNotice::msg_id`][msg_id]
///
/// [msg_id]: ./struct.UserNotice.html#method.msg_id
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NoticeType<'t> {
    /// This was a subscription notice
    Sub,
    /// This was a re-subscription notice
    Resub,
    /// This was a gifted subscription
    SubGift,
    /// This was an anonymous gifted subscription
    AnonSubGift,
    /// This was a mystery gift for the subscription
    SubMysteryGift,
    /// Gift for a paid upgrade
    GiftPaidUpgrade,
    /// A reward gift
    RewardGift,
    /// An anonymous gift for paid upgrade
    AnonGiftPaidUpgrade,
    /// A raid
    Raid,
    /// A canceled raid
    Unraid,
    /// A ritual
    Ritual,
    /// A the tier that the bits were part of
    BitsBadgeTier,
    /// An unknown notice type (a catch-all)
    Unknown(Cow<'t, str>),
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for UserNotice<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("USERNOTICE")?;
        let channel = msg.expect_arg(0)?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel,
            message: msg.data.clone(),
        })
    }
}

impl<'t> AsOwned for UserNotice<'t> {
    type Owned = UserNotice<'static>;
    fn as_owned(&self) -> Self::Owned {
        UserNotice {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
            message: self.message.as_owned(),
        }
    }
}
