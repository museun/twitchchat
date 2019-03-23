use super::*;

/// Announces Twitch-specific events to the channel (e.g., a user's subscription notification).
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserNotice {
    /// IRC tags
    pub tags: Tags,
    /// The channel this event is for
    pub channel: String,
    /// Extra data provided by twitch
    pub message: Option<String>,
}

impl UserNotice {
    /// The channel this event is for
    pub fn channel(&self) -> &str {
        &self.channel
    }
    /// The message. This is omitted if the user did not enter a message.
    pub fn message(&self) -> Option<&str> {
        // TODO technically this won't ever be in the tags
        self.get("message")
            .or_else(|| self.message.as_ref().map(|s| s.as_ref()))
    }
}

impl UserNotice {
    /// List of badges attached to this message
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<TwitchColor> {
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
    pub fn id(&self) -> Option<&str> {
        self.get("id")
    }
    /// The name of the user who sent the notice.
    pub fn login(&self) -> Option<&str> {
        self.get("login")
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
    /// (Sent only on sub, resub) The total number of months the user has
    /// subscribed. This is the same as msg-param-months but sent for different
    /// types of user notices.
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
    pub fn room_id(&self) -> Option<u64> {
        self.get_parsed("room-id")
    }
    /// The message printed in chat along with this notice.
    pub fn system_msg(&self) -> String {
        self.get("system-msg")
            .map(|s| {
                s.replace("\\s", " ")
                    .replace("\\r", "\r")
                    .replace("\\n", "\n")
                    .replace("\\\\", "\\")
                    .replace("\\:", ":")
            })
            .unwrap()
    }
    /// Timestamp of when the notice was sent
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.get_parsed("tmi-sent-ts")
    }
    /// The id of the user who sent the notice
    pub fn user_id(&self) -> Option<u64> {
        self.get_parsed("user-id")
    }
}

impl Tag for UserNotice {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}

/// A paid subscription to the channel
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
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

/// The type of message this User Notice is
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// -- not documented
    SubMysteryGift,
    /// -- not documented
    GiftPaidUpgrade,
    /// -- not documented
    RewardGift,
    /// -- not documented
    AnonGiftPaidUpgrade,
    /// Enables:
    /// - msg-param-displayName,
    /// - msg-param-login,
    /// - msg-param-viewerCount
    Raid,
    /// -- not documented
    Unraid,
    /// Enables:
    /// - msg-param-ritual-name
    Ritual,
    /// -- not documented
    BitsBadgeTier,
    /// An unknown message type
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_msg_unescape() {
        let input = [
r#"@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=ronni;emotes=;id=db25007f-7a18-43eb-9379-80131e44d633;login=ronni;mod=0;msg-id=resub;msg-param-cumulative-months=6;msg-param-streak-months=2;msg-param-should-share-streak=1;msg-param-sub-plan=Prime;msg-param-sub-plan-name=Prime;room-id=1337;subscriber=1;system-msg=ronni\shas\ssubscribed\sfor\s6\smonths!;tmi-sent-ts=1507246572675;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #dallas :Great stream -- keep it up!"#,
r#"@badges=broadcaster/1,subscriber/6;color=;display-name=qa_subs_partner;emotes=;flags=;id=b1818e3c-0005-490f-ad0a-804957ddd760;login=qa_subs_partner;mod=0;msg-id=anonsubgift;msg-param-months=3;msg-param-recipient-display-name=TenureCalculator;msg-param-recipient-id=135054130;msg-param-recipient-user-name=tenurecalculator;msg-param-sub-plan-name=t111;msg-param-sub-plan=1000;room-id=196450059;subscriber=1;system-msg=An\sanonymous\suser\sgifted\sa\sTier\s1\ssub\sto\sTenureCalculator!\s;tmi-sent-ts=1542063432068;turbo=0;user-id=196450059;user-type= :tmi.twitch.tv USERNOTICE #qa_subs_partner"#,
r#"@badges=turbo/1;color=#9ACD32;display-name=TestChannel;emotes=;id=3d830f12-795c-447d-af3c-ea05e40fbddb;login=testchannel;mod=0;msg-id=raid;msg-param-displayName=TestChannel;msg-param-login=testchannel;msg-param-viewerCount=15;room-id=56379257;subscriber=0;system-msg=15\sraiders\sfrom\sTestChannel\shave\sjoined\n!;tmi-sent-ts=1507246572675;tmi-sent-ts=1507246572675;turbo=1;user-id=123456;user-type= :tmi.twitch.tv USERNOTICE #othertestchannel"#,
r#"@badges=;color=;display-name=SevenTest1;emotes=30259:0-6;id=37feed0f-b9c7-4c3a-b475-21c6c6d21c3d;login=seventest1;mod=0;msg-id=ritual;msg-param-ritual-name=new_chatter;room-id=6316121;subscriber=0;system-msg=Seventoes\sis\snew\shere!;tmi-sent-ts=1508363903826;turbo=0;user-id=131260580;user-type= :tmi.twitch.tv USERNOTICE #seventoes :HeyGuys"#,
        ];

        let output = [
            "ronni has subscribed for 6 months!",
            "An anonymous user gifted a Tier 1 sub to TenureCalculator! ",
            "15 raiders from TestChannel have joined\n!",
            "Seventoes is new here!",
        ];

        for (input, &output) in input.iter().zip(output.iter()) {
            let msg = crate::irc::types::Message::parse(input).unwrap();
            if let crate::twitch::Message::UserNotice(msg @ UserNotice { .. }) =
                parse(&msg).unwrap()
            {
                assert_eq!(msg.system_msg(), output);
            }
        }
    }
}
