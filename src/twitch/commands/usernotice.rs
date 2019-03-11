use super::*;

/// Announces Twitch-specific events to the channel (e.g., a user's subscription notification).
#[derive(Debug, PartialEq, Clone)]
pub struct UserNotice {
    pub(crate) tags: Tags,
    /// THe channel this event is for
    pub channel: String,
    /// Extra data provided by twitch
    pub message: Option<String>,
}

impl UserNotice {
    /// List of badges attached to this message
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<Color> {
        self.get("color").map(RGB::from_hex).map(Into::into)
    }
    /// The display name of the user, if set
    pub fn display_name(&self) -> Option<&str> {
        self.get("display-name")
    }
    /// List of emotes found in the message body.
    pub fn emotes(&self) -> Vec<Emotes> {
        emotes(self.get("emotes").unwrap_or_default())
    }
    /// A unique id (UUID) attached to this message (used for Localization)
    pub fn id(&self) -> Option<uuid::Uuid> {
        self.get_parsed("id")
    }
    /// The name of the user who sent the notice.
    pub fn login(&self) -> Option<&str> {
        self.get("login")
    }
    /// The message. This is omitted if the user did not enter a message.
    pub fn message(&self) -> Option<&str> {
        self.get("message")
    }
    /// Whether this user is a moderator
    pub fn moderator(&self) -> bool {
        self.get_as_bool("mod")
    }
    /// The type of notice, see NoticeType
    pub fn msg_id(&self) -> NoticeType {
        self.get("msg-id")
            .map(|k| match k {
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
                e => NoticeType::Unknown(e.to_string()),
            })
            .unwrap()
    }
    // TODO maybe parse this into a struct
    //  (Sent only on sub, resub) The total number of months the user has
    //  subscribed. This is the same as msg-param-months but sent for different
    //  types of user notices.
    pub fn msg_param_cumulative_months(&self) -> Option<u64> {
        self.get_parsed("msg-param-cumulative-months")
    }
    /// (Sent only on raid) The display name of the source user raiding this
    /// channel.
    pub fn msg_param_display_name(&self) -> Option<&str> {
        self.get("msg-param-displayName")
    }
    /// (Sent on only raid) The name of the source user raiding this channel.
    pub fn msg_param_login(&self) -> Option<&str> {
        self.get("msg-param-login")
    }
    /// (Sent only on subgift, anonsubgift) The total number of months the user
    /// has subscribed. This is the same as msg-param-cumulative-months but sent
    /// for different types of user notices.
    pub fn msg_param_months(&self) -> Option<u64> {
        self.get_parsed("msg-param-months")
    }
    /// (Sent only on subgift, anonsubgift) The display name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_display_name(&self) -> Option<&str> {
        self.get("msg-param-recipient-display-name")
    }
    /// (Sent only on subgift, anonsubgift) The user ID of the subscription gift
    /// recipient.
    pub fn msg_param_recipient_id(&self) -> Option<u64> {
        self.get_parsed("msg-param-recipient-id")
    }
    /// (Sent only on subgift, anonsubgift) The user name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_user_name(&self) -> Option<&str> {
        self.get("msg-param-recipient-user-name")
    }
    /// (Sent only on sub, resub) Boolean indicating whether users want their
    /// streaks to be shared. TODO option?
    pub fn msg_param_should_share_streak(&self) -> bool {
        self.get_as_bool("msg-param-should-share-streak")
    }
    /// (Sent only on sub, resub) The number of consecutive months the user has
    /// subscribed. This is 0 if msg-param-should-share-streak is 0.
    pub fn msg_param_streak_months(&self) -> Option<u64> {
        self.get_parsed("msg-param-streak-months")
    }
    /// (Sent only on sub, resub, subgift, anonsubgift) The type of subscription
    /// plan being used. Valid values: Prime, 1000, 2000, 3000. 1000, 2000, and
    /// 3000 refer to the first, second, and third levels of paid subscriptions,
    /// respectively (currently $4.99, $9.99, and $24.99).
    pub fn msg_param_sub_plan(&self) -> Option<SubPlan> {
        self.get("msg-param-sub-plan").and_then(|k| {
            let plan = match k {
                "Prime" => SubPlan::Prime,
                "1000" => SubPlan::Tier1,
                "2000" => SubPlan::Tier2,
                "3000" => SubPlan::Tier3,
                _ => return None, // TODO warn on this
            };
            Some(plan)
        })
    }
    /// (Sent only on sub, resub, subgift, anonsubgift) The display name of the
    /// subscription plan. This may be a default name or one created by the
    /// channel owner.
    pub fn msg_param_sub_plan_name(&self) -> Option<&str> {
        self.get("msg-param-sub-plan-name")
    }
    /// (Sent only on raid) The number of viewers watching the source channel
    /// raiding this channel.
    pub fn msg_param_viewer_count(&self) -> Option<u64> {
        self.get_parsed("msg-param-viewerCount")
    }
    /// (Sent only on ritual) The name of the ritual this notice is for. Valid
    /// value: new_chatter.
    pub fn msg_param_ritual_name(&self) -> Option<&str> {
        self.get("msg-param-ritual-name")
    }
    /// The id for the room for this notice
    pub fn room_id(&self) -> u64 {
        self.get_parsed("room-id").unwrap()
    }
    /// The message printed in chat along with this notice.
    pub fn system_msg(&self) -> &str {
        self.get("system-msg").unwrap()
    }
    /// Timestamp of when the notice was sent
    pub fn tmi_sent_ts(&self) -> u64 {
        self.get_parsed("tmi-sent-ts").unwrap()
    }
    /// The id of the user who sent the notice
    pub fn user_id(&self) -> u64 {
        self.get_parsed("user-id").unwrap()
    }
}

impl Tag for UserNotice {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}

/// A paid subscription to the channel
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
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

/// The type of message this User Notice is
#[derive(Debug, PartialEq, Clone)]
pub enum NoticeType {
    /// Enables:
    /// - msg-param-cumulative-months,
    /// - msg-param-should-share-streak,
    /// - msg-param-streak-months,
    /// - msg-param-sub-plan,
    /// - msg-param-sub-plan-name
    Sub,
    /// Enables:
    /// - msg-param-cumulative-months,
    /// - msg-param-should-share-streak,
    /// - msg-param-streak-months,
    /// - msg-param-sub-plan,
    /// - msg-param-sub-plan-name
    Resub,
    /// Enables:
    /// - msg-param-months,
    /// - msg-param-recipient-display-name,
    /// - msg-param-recipient-id,
    /// - msg-param-recipient-user-name,
    /// - msg-param-sub-plan,
    /// - msg-param-sub-plan-name
    SubGift,
    /// Enables:
    /// - msg-param-months
    /// - msg-param-recipient-display-name
    /// - msg-param-recipient-id
    /// - msg-param-recipient-user-name
    /// - msg-param-sub-plan
    /// - msg-param-sub-plan-name
    AnonSubGift,
    SubMysteryGift,
    GiftPaidUpgrade,
    RewardGift,
    AnonGiftPaidUpgrade,
    /// Enables:
    /// - msg-param-displayName,
    /// - msg-param-login,
    /// - msg-param-viewerCount
    Raid,
    Unraid,
    /// Enables:
    /// - msg-param-ritual-name
    Ritual,
    BitsBadgeTier,
    /// An unknown message type
    Unknown(String),
}
