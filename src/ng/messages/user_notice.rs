use crate::{
    color::Color,
    ng::{FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, TagIndices, Tags, Validator},
    parse_badges, parse_emotes, Badge, BadgeInfo, Emotes,
};

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
//
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub enum SubPlan {
    Prime,
    Tier1,
    Tier2,
    Tier3,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Hash)]
//
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub enum NoticeType<'t> {
    Sub,
    Resub,
    SubGift,
    AnonSubGift,
    SubMysteryGift,
    GiftPaidUpgrade,
    RewardGift,
    AnonGiftPaidUpgrade,
    Raid,
    Unraid,
    Ritual,
    BitsBadgeTier,
    Unknown(&'t str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserNotice<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    channel: StrIndex,
    message: Option<StrIndex>,
}

impl<'t> UserNotice<'t> {
    raw!();
    tags!();
    str_field!(channel);
    opt_str_field!(message);

    pub fn badge_info(&'t self) -> Vec<BadgeInfo<'t>> {
        self.tags()
            .get("badge-info")
            .map(parse_badges)
            .unwrap_or_default()
    }

    pub fn badges(&'t self) -> Vec<Badge<'t>> {
        self.tags()
            .get("badges")
            .map(parse_badges)
            .unwrap_or_default()
    }

    pub fn color(&self) -> Option<Color> {
        self.tags().get_parsed("color")
    }

    pub fn display_name(&self) -> Option<&str> {
        self.tags().get("display-name")
    }

    pub fn emotes(&self) -> Vec<Emotes> {
        self.tags()
            .get("emotes")
            .map(parse_emotes)
            .unwrap_or_default()
    }

    pub fn id(&self) -> Option<&str> {
        self.tags().get("id")
    }

    pub fn login(&self) -> Option<&str> {
        self.tags().get("login")
    }

    pub fn is_moderator(&self) -> bool {
        self.tags().get_as_bool("mod")
    }

    pub fn msg_id(&'t self) -> Option<NoticeType<'t>> {
        let kind = self.tags().get("msg-id")?;
        match kind {
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
            kind => NoticeType::Unknown(kind),
        }
        .into()
    }

    pub fn room_id(&self) -> Option<u64> {
        self.tags().get_parsed("room-id")
    }

    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    pub fn user_id(&self) -> Option<u64> {
        self.tags().get_parsed("user-id")
    }

    pub fn system_msg(&self) -> Option<String> {
        self.tags()
            .get("system-msg")?
            .replace("\\s", " ")
            .replace("\\r", "\r")
            .replace("\\n", "\n")
            .replace("\\\\", "\\")
            .replace("\\:", ":")
            .into()
    }

    pub fn msg_param_cumulative_months(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-cumulative-months")
    }

    pub fn msg_param_display_name(&self) -> Option<&str> {
        self.tags().get("msg-param-displayName")
    }

    pub fn msg_param_login(&self) -> Option<&str> {
        self.tags().get("msg-param-login")
    }

    pub fn msg_param_months(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-months")
    }

    pub fn msg_param_promo_gift_total(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-promo-gift-total")
    }

    pub fn msg_param_promo_name(&self) -> Option<&str> {
        self.tags().get("msg-param-promo-name")
    }

    pub fn msg_param_recipient_display_name(&self) -> Option<&str> {
        self.tags().get("msg-param-recipient-display-name")
    }

    pub fn msg_param_recipient_id(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-recipient-id")
    }

    pub fn msg_param_recipient_user_name(&self) -> Option<&str> {
        self.tags().get("msg-param-recipient-user-name")
    }

    pub fn msg_param_sender_login(&self) -> Option<&str> {
        self.tags().get("msg-param-sender-login")
    }

    pub fn msg_param_sender_name(&self) -> Option<&str> {
        self.tags().get("msg-param-sender-name")
    }

    pub fn msg_param_should_share_streak(&self) -> Option<bool> {
        self.tags().get_parsed("msg-param-should-share-streak")
    }

    pub fn msg_param_streak_months(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-streak-months")
    }

    pub fn msg_param_sub_plan(&self) -> Option<SubPlan> {
        self.tags().get("msg-param-sub-plan").and_then(|s| {
            match s {
                "Prime" => SubPlan::Prime,
                "Tier1" => SubPlan::Tier1,
                "Tier2" => SubPlan::Tier2,
                "Tier3" => SubPlan::Tier3,
                _ => return None, // TODO warn on this?
            }
            .into()
        })
    }

    pub fn msg_param_sub_plan_name(&self) -> Option<&str> {
        self.tags().get("msg-param-sub-plan-name")
    }

    pub fn msg_param_viewer_count(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-viewerCount")
    }

    pub fn msg_param_ritual_name(&self) -> Option<&str> {
        self.tags().get("msg-param-ritual-name")
    }

    pub fn msg_param_threshold(&self) -> Option<u64> {
        self.tags().get_parsed("msg-param-threshold")
    }
}

impl<'t> FromIrcMessage<'t> for UserNotice<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::USERNOTICE)?;

        let this = Self {
            channel: msg.expect_arg_index(0)?,
            message: msg.data,
            tags: msg.parse_tags(),
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(UserNotice {
    raw,
    tags,
    channel,
    message,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn user_notice_serde() {
        let input = &[
            ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n",
            ":tmi.twitch.tv USERNOTICE #museun\r\n",
            "@badge-info=subscriber/8;badges=subscriber/6,bits/100;color=#59517B;display-name=lllAirJordanlll;emotes=;flags=;id=3198b02c-eaf4-4904-9b07-eb1b2b12ba50;login=lllairjordanlll;mod=0;msg-id=resub;msg-param-cumulative-months=8;msg-param-months=0;msg-param-should-share-streak=0;msg-param-sub-plan-name=Channel\\sSubscription\\s(giantwaffle);msg-param-sub-plan=1000;room-id=22552479;subscriber=1;system-msg=lllAirJordanlll\\ssubscribed\\sat\\sTier\\s1.\\sThey\'ve\\ssubscribed\\sfor\\s8\\smonths!;tmi-sent-ts=1580932171144;user-id=44979519;user-type= :tmi.twitch.tv USERNOTICE #giantwaffle\r\n",
        ];

        for input in input {
            crate::ng::serde::round_trip_json::<UserNotice>(input);
        }
    }

    #[test]
    fn user_notice_message() {
        let input = ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n";

        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(
                msg.message().unwrap(),
                "This room is no longer in slow mode."
            );
        }
    }

    #[test]
    fn user_notice() {
        let input = ":tmi.twitch.tv USERNOTICE #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.message(), None);
        }
    }

    #[test]
    fn user_notice_unknown() {
        let input = "@badge-info=subscriber/8;badges=subscriber/6,bits/100;color=#59517B;display-name=lllAirJordanlll;emotes=;flags=;id=3198b02c-eaf4-4904-9b07-eb1b2b12ba50;login=lllairjordanlll;mod=0;msg-id=resub;msg-param-cumulative-months=8;msg-param-months=0;msg-param-should-share-streak=0;msg-param-sub-plan-name=Channel\\sSubscription\\s(giantwaffle);msg-param-sub-plan=1000;room-id=22552479;subscriber=1;system-msg=lllAirJordanlll\\ssubscribed\\sat\\sTier\\s1.\\sThey\'ve\\ssubscribed\\sfor\\s8\\smonths!;tmi-sent-ts=1580932171144;user-id=44979519;user-type= :tmi.twitch.tv USERNOTICE #giantwaffle\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#giantwaffle");
            assert_eq!(msg.tags().is_empty(), false);
        }
    }
}
