use super::expect::Expect as _;
use super::*;
use crate::{AsOwned, Parse};

impl<'a: 't, 't> Parse<&'a Message<'t>> for Raw<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        Ok(msg.clone())
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for AllCommands<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        let out = match &*msg.command {
            "001" => IrcReady::parse(msg)?.into(),
            "353" => Names::parse(msg)?.into(),
            "366" => Names::parse(msg)?.into(),
            "376" => Ready::parse(msg)?.into(),
            "CAP" => Cap::parse(msg)?.into(),
            "CLEARCHAT" => ClearChat::parse(msg)?.into(),
            "CLEARMSG" => ClearMsg::parse(msg)?.into(),
            "GLOBALUSERSTATE" => GlobalUserState::parse(msg)?.into(),
            "HOSTARGET" => HostTarget::parse(msg)?.into(),
            "JOIN" => Join::parse(msg)?.into(),
            "MODE" => Mode::parse(msg)?.into(),
            "NOTICE" => Notice::parse(msg)?.into(),
            "PART" => Part::parse(msg)?.into(),
            "PING" => Ping::parse(msg)?.into(),
            "PONG" => Pong::parse(msg)?.into(),
            "PRIVMSG" => Privmsg::parse(msg)?.into(),
            "RECONNECT" => Reconnect::parse(msg)?.into(),
            "ROOMSTATE" => RoomState::parse(msg)?.into(),
            "USERNOTICE" => UserNotice::parse(msg)?.into(),
            "USERSTATE" => UserState::parse(msg)?.into(),
            _ => msg.clone().into(),
        };
        Ok(out)
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for RoomState<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("ROOMSTATE")?;
        Ok(Self {
            channel: msg.expect_arg(0)?,
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for RoomState<'t> {
    type Owned = RoomState<'static>;
    fn as_owned(&self) -> Self::Owned {
        RoomState {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
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

impl<'a: 't, 't> Parse<&'a Message<'t>> for Names<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        let kind = match &*msg.command {
            "353" => {
                let users = msg.expect_data()?.split_whitespace();
                let users = users.map(Cow::Borrowed).collect();
                NamesKind::Start { users }
            }
            "366" => NamesKind::End,
            unknown => {
                return Err(InvalidMessage::InvalidCommand {
                    expected: "353 or 366".to_string(),
                    got: unknown.to_string(),
                })
            }
        };
        let name = msg.expect_arg(0)?;
        let channel = match msg.expect_arg(1)? {
            d if d == "=" => msg.expect_arg(2)?,
            channel => channel,
        };
        Ok(Self {
            name,
            channel,
            kind,
        })
    }
}

impl<'t> AsOwned for Names<'t> {
    type Owned = Names<'static>;
    fn as_owned(&self) -> Self::Owned {
        Names {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
            kind: self.kind.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for GlobalUserState<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("GLOBALUSERSTATE")?;
        let user_id = msg
            .tags
            .get("user-id")
            .cloned()
            .expect("user-id attached to message");
        let display_name = msg.tags.get("display-name").cloned();
        let color = msg
            .tags
            .get("color")
            .and_then(|s| s.parse().ok())
            .clone()
            .unwrap_or_default();
        let emote_sets = msg
            .tags
            .get("emotes-set")
            .map(|s| s.split(',').map(Into::into).collect())
            .unwrap_or_else(|| vec!["0".into()]);
        let badges = msg
            .tags
            .get("badges")
            .map(|s| s.split(',').filter_map(crate::Badge::parse).collect())
            .unwrap_or_default();
        Ok(Self {
            user_id,
            display_name,
            color,
            emote_sets,
            badges,
        })
    }
}

impl<'t> AsOwned for GlobalUserState<'t> {
    type Owned = GlobalUserState<'static>;
    fn as_owned(&self) -> Self::Owned {
        GlobalUserState {
            user_id: self.user_id.as_owned(),
            display_name: self.display_name.as_owned(),
            color: self.color.as_owned(),
            emote_sets: self.emote_sets.as_owned(),
            badges: self.badges.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for HostTarget<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("HOSTTARGET")?;
        let source = msg.expect_arg(0)?;
        let (kind, viewers) = if let Ok(target) = msg.expect_arg(1) {
            let viewers = msg.expect_arg(2).ok().and_then(|data| data.parse().ok());
            (HostTargetKind::Start { target }, viewers)
        } else {
            let data = msg.expect_data()?;
            if !data.starts_with('-') {
                return Err(InvalidMessage::ExpectedData);
            }
            let viewers = data.get(2..).and_then(|s| s.parse().ok());
            (HostTargetKind::End, viewers)
        };
        Ok(Self {
            source,
            kind,
            viewers,
        })
    }
}

impl<'t> AsOwned for HostTarget<'t> {
    type Owned = HostTarget<'static>;
    fn as_owned(&self) -> Self::Owned {
        HostTarget {
            source: self.source.as_owned(),
            viewers: self.viewers.as_owned(),
            kind: self.kind.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Cap<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("CAP")?;
        let acknowledged = msg.expect_arg(1)? == "ACK";
        let capability = msg.expect_data()?.clone();
        Ok(Self {
            capability,
            acknowledged,
        })
    }
}

impl<'t> AsOwned for Cap<'t> {
    type Owned = Cap<'static>;
    fn as_owned(&self) -> Self::Owned {
        Cap {
            capability: self.capability.as_owned(),
            acknowledged: self.acknowledged.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for ClearChat<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("CLEARCHAT")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            name: msg.expect_data().ok().cloned(),
        })
    }
}

impl<'t> AsOwned for ClearChat<'t> {
    type Owned = ClearChat<'static>;
    fn as_owned(&self) -> Self::Owned {
        ClearChat {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
            name: self.name.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for ClearMsg<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("CLEARMSG")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            message: msg.expect_data().ok().cloned(),
        })
    }
}

impl<'t> AsOwned for ClearMsg<'t> {
    type Owned = ClearMsg<'static>;
    fn as_owned(&self) -> Self::Owned {
        ClearMsg {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
            message: self.message.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for IrcReady<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("001")?;
        msg.expect_arg(0).map(|nickname| Self { nickname })
    }
}

impl<'t> AsOwned for IrcReady<'t> {
    type Owned = IrcReady<'static>;
    fn as_owned(&self) -> Self::Owned {
        IrcReady {
            nickname: self.nickname.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Join<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("JOIN")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

impl<'t> AsOwned for Join<'t> {
    type Owned = Join<'static>;
    fn as_owned(&self) -> Self::Owned {
        Join {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Mode<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("MODE")?;
        let channel = msg.expect_arg(0)?;
        let status = match msg.expect_arg(1)?.chars().next().unwrap() {
            '+' => ModeStatus::Gained,
            '-' => ModeStatus::Lost,
            _ => unreachable!(),
        };
        let name = msg.expect_arg(2)?;
        Ok(Self {
            channel,
            status,
            name,
        })
    }
}

impl<'t> AsOwned for Mode<'t> {
    type Owned = Mode<'static>;
    fn as_owned(&self) -> Self::Owned {
        Mode {
            channel: self.channel.as_owned(),
            status: self.status.as_owned(),
            name: self.name.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Notice<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("NOTICE")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            message: msg.expect_data()?.clone(),
        })
    }
}

impl<'t> AsOwned for Notice<'t> {
    type Owned = Notice<'static>;
    fn as_owned(&self) -> Self::Owned {
        Notice {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
            message: self.message.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Part<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PART")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

impl<'t> AsOwned for Part<'t> {
    type Owned = Part<'static>;
    fn as_owned(&self) -> Self::Owned {
        Part {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Ping<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PING")?;
        msg.expect_data().map(|token| Self {
            token: token.clone(),
        })
    }
}

impl<'t> AsOwned for Ping<'t> {
    type Owned = Ping<'static>;
    fn as_owned(&self) -> Self::Owned {
        Ping {
            token: self.token.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Pong<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PONG")?;
        msg.expect_data().map(|token| Self {
            token: token.clone(),
        })
    }
}

impl<'t> AsOwned for Pong<'t> {
    type Owned = Pong<'static>;
    fn as_owned(&self) -> Self::Owned {
        Pong {
            token: self.token.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Privmsg<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PRIVMSG")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
            data: msg.expect_data()?.clone(),
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for Privmsg<'t> {
    type Owned = Privmsg<'static>;
    fn as_owned(&self) -> Self::Owned {
        Privmsg {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
            data: self.data.as_owned(),
            tags: self.tags.as_owned(),
        }
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Ready<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("376")?;
        msg.expect_arg(0).map(|username| Self { username })
    }
}

impl<'t> AsOwned for Ready<'t> {
    type Owned = Ready<'static>;
    fn as_owned(&self) -> Self::Owned {
        Ready {
            username: self.username.as_owned(),
        }
    }
}

impl AsOwned for Reconnect {
    type Owned = Reconnect;
    fn as_owned(&self) -> Self::Owned {
        Reconnect {}
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Reconnect {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("RECONNECT").map(|_| Self {})
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for UserState<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("USERSTATE")?;
        msg.expect_arg(0).map(|channel| Self {
            channel,
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for UserState<'t> {
    type Owned = UserState<'static>;
    fn as_owned(&self) -> Self::Owned {
        UserState {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}

impl<'t> MessageId<'t> {
    pub(super) fn parse(input: &'t str) -> MessageId<'t> {
        use MessageId::*;
        match input {
            "already_banned" => AlreadyBanned,
            "already_emote_only_off" => AlreadyEmoteOnlyOff,
            "already_emote_only_on" => AlreadyEmoteOnlyOn,
            "already_r9k_off" => AlreadyR9kOff,
            "already_r9k_on" => AlreadyR9kOn,
            "already_subs_off" => AlreadySubsOff,
            "already_subs_on" => AlreadySubsOn,
            "bad_ban_admin" => BadBanAdmin,
            "bad_ban_anon" => BadBanAnon,
            "bad_ban_broadcaster" => BadBanBroadcaster,
            "bad_ban_global_mod" => BadBanGlobalMod,
            "bad_ban_mod" => BadBanMod,
            "bad_ban_self" => BadBanSelf,
            "bad_ban_staff" => BadBanStaff,
            "bad_commercial_error" => BadCommercialError,
            "bad_delete_message_broadcaster" => BadDeleteMessageBroadcaster,
            "bad_delete_message_mod" => BadDeleteMessageMod,
            "bad_host_error" => BadHostError,
            "bad_host_hosting" => BadHostHosting,
            "bad_host_rate_exceeded" => BadHostRateExceeded,
            "bad_host_rejected" => BadHostRejected,
            "bad_host_self" => BadHostSelf,
            "bad_marker_client" => BadMarkerClient,
            "bad_mod_banned" => BadModBanned,
            "bad_mod_mod" => BadModMod,
            "bad_slow_duration" => BadSlowDuration,
            "bad_timeout_admin" => BadTimeoutAdmin,
            "bad_timeout_anon" => BadTimeoutAnon,
            "bad_timeout_broadcaster" => BadTimeoutBroadcaster,
            "bad_timeout_duration" => BadTimeoutDuration,
            "bad_timeout_global_mod" => BadTimeoutGlobalMod,
            "bad_timeout_mod" => BadTimeoutMod,
            "bad_timeout_self" => BadTimeoutSelf,
            "bad_timeout_staff" => BadTimeoutStaff,
            "bad_unban_no_ban" => BadUnbanNoBan,
            "bad_unhost_error" => BadUnhostError,
            "bad_unmod_mod" => BadUnmodMod,
            "ban_success" => BanSuccess,
            "cmds_available" => CmdsAvailable,
            "color_changed" => ColorChanged,
            "commercial_success" => CommercialSuccess,
            "delete_message_success" => DeleteMessageSuccess,
            "emote_only_off" => EmoteOnlyOff,
            "emote_only_on" => EmoteOnlyOn,
            "followers_off" => FollowersOff,
            "followers_on" => FollowersOn,
            // ERROR: docs have this as 'followers_onzero'
            "followers_on_zero" => FollowersOnZero,
            "host_off" => HostOff,
            "host_on" => HostOn,
            "host_success" => HostSuccess,
            "host_success_viewers" => HostSuccessViewers,
            "host_target_went_offline" => HostTargetWentOffline,
            "hosts_remaining" => HostsRemaining,
            "invalid_user" => InvalidUser,
            "mod_success" => ModSuccess,
            "msg_banned" => MsgBanned,
            "msg_bad_characters" => MsgBadCharacters,
            "msg_channel_blocked" => MsgChannelBlocked,
            "msg_channel_suspended" => MsgChannelSuspended,
            "msg_duplicate" => MsgDuplicate,
            "msg_emoteonly" => MsgEmoteonly,
            "msg_facebook" => MsgFacebook,
            "msg_followersonly" => MsgFollowersonly,
            "msg_followersonly_followed" => MsgFollowersonlyFollowed,
            "msg_followersonly_zero" => MsgFollowersonlyZero,
            "msg_r9k" => MsgR9k,
            "msg_ratelimit" => MsgRatelimit,
            "msg_rejected" => MsgRejected,
            "msg_rejected_mandatory" => MsgRejectedMandatory,
            "msg_room_not_found" => MsgRoomNotFound,
            "msg_slowmode" => MsgSlowmode,
            "msg_subsonly" => MsgSubsonly,
            "msg_suspended" => MsgSuspended,
            "msg_timedout" => MsgTimedout,
            "msg_verified_email" => MsgVerifiedEmail,
            "no_help" => NoHelp,
            "no_mods" => NoMods,
            "not_hosting" => NotHosting,
            "no_permission" => NoPermission,
            "r9k_off" => R9kOff,
            "r9k_on" => R9kOn,
            "raid_error_already_raiding" => RaidErrorAlreadyRaiding,
            "raid_error_forbidden" => RaidErrorForbidden,
            "raid_error_self" => RaidErrorSelf,
            "raid_error_too_many_viewers" => RaidErrorTooManyViewers,
            "raid_error_unexpected" => RaidErrorUnexpected,
            "raid_notice_mature" => RaidNoticeMature,
            "raid_notice_restricted_chat" => RaidNoticeRestrictedChat,
            "room_mods" => RoomMods,
            "slow_off" => SlowOff,
            "slow_on" => SlowOn,
            "subs_off" => SubsOff,
            "subs_on" => SubsOn,
            "timeout_no_timeout" => TimeoutNoTimeout,
            "timeout_success" => TimeoutSuccess,
            "tos_ban" => TosBan,
            "turbo_only_color" => TurboOnlyColor,
            "unban_success" => UnbanSuccess,
            "unmod_success" => UnmodSuccess,
            "unraid_error_no_active_raid" => UnraidErrorNoActiveRaid,
            "unraid_error_unexpected" => UnraidErrorUnexpected,
            "unraid_success" => UnraidSuccess,
            "unrecognized_cmd" => UnrecognizedCmd,
            "unsupported_chatrooms_cmd" => UnsupportedChatroomsCmd,
            "untimeout_banned" => UntimeoutBanned,
            "untimeout_success" => UntimeoutSuccess,
            "usage_ban" => UsageBan,
            "usage_clear" => UsageClear,
            "usage_color" => UsageColor,
            "usage_commercial" => UsageCommercial,
            "usage_disconnect" => UsageDisconnect,
            "usage_emote_only_off" => UsageEmoteOnlyOff,
            "usage_emote_only_on" => UsageEmoteOnlyOn,
            "usage_followers_off" => UsageFollowersOff,
            "usage_followers_on" => UsageFollowersOn,
            "usage_help" => UsageHelp,
            "usage_host" => UsageHost,
            "usage_marker" => UsageMarker,
            "usage_me" => UsageMe,
            "usage_mod" => UsageMod,
            "usage_mods" => UsageMods,
            "usage_r9k_off" => UsageR9kOff,
            "usage_r9k_on" => UsageR9kOn,
            "usage_raid" => UsageRaid,
            "usage_slow_off" => UsageSlowOff,
            "usage_slow_on" => UsageSlowOn,
            "usage_subs_off" => UsageSubsOff,
            "usage_subs_on" => UsageSubsOn,
            "usage_timeout" => UsageTimeout,
            "usage_unban" => UsageUnban,
            "usage_unhost" => UsageUnhost,
            "usage_unmod" => UsageUnmod,
            "usage_unraid" => UsageUnraid,
            "usage_untimeout" => UsageUntimeout,
            "whisper_banned" => WhisperBanned,
            "whisper_banned_recipient" => WhisperBannedRecipient,
            "whisper_invalid_args" => WhisperInvalidArgs,
            "whisper_invalid_login" => WhisperInvalidLogin,
            "whisper_invalid_self" => WhisperInvalidSelf,
            "whisper_limit_per_min" => WhisperLimitPerMin,
            "whisper_limit_per_sec" => WhisperLimitPerSec,
            "whisper_restricted" => WhisperRestricted,
            "whisper_restricted_recipient" => WhisperRestrictedRecipient,
            _ => Unknown(Cow::Borrowed(input)),
        }
    }
}
