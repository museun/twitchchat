use super::expect::Expect as _;
use super::*;
use crate::{AsOwned, Parse};

parse! {
    bare Raw {
        raw,
        tags,
        prefix,
        command,
        args,
        data
    } => |msg: &'t Message<'t>| {
        Ok(msg.clone())
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for AllCommands<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        let out = match &*msg.command {
            "001" => IrcReady::parse(msg)?.into(),
            "PING" => Ping::parse(msg)?.into(),
            "PONG" => Pong::parse(msg)?.into(),
            "376" => Ready::parse(msg)?.into(),
            "353" => Names::parse(msg)?.into(),
            "366" => Names::parse(msg)?.into(),
            "JOIN" => Join::parse(msg)?.into(),
            "PART" => Part::parse(msg)?.into(),
            "PRIVMSG" => Privmsg::parse(msg)?.into(),
            "CAP" => Cap::parse(msg)?.into(),
            "HOSTTARGET" => HostTarget::parse(msg)?.into(),
            "GLOBALUSERSTATE" => GlobalUserState::parse(msg)?.into(),
            "NOTICE" => Notice::parse(msg)?.into(),
            "CLEARCHAT" => ClearChat::parse(msg)?.into(),
            "CLEARMSG" => ClearMsg::parse(msg)?.into(),
            "RECONNECT" => Reconnect::parse(msg)?.into(),
            "ROOMSTATE" => RoomState::parse(msg)?.into(),
            "USERNOTICE" => UserNotice::parse(msg)?.into(),
            "USERSTATE" => UserState::parse(msg)?.into(),
            "MODE" => Mode::parse(msg)?.into(),
            "WHISPER" => Whisper::parse(msg)?.into(),
            _ => msg.clone().into(),
        };
        Ok(out)
    }
}

parse! {
    RoomState { tags, channel } => |msg: &'t Message<'t>| {
        msg.expect_command("ROOMSTATE")?;
        Ok(Self {
            channel: msg.expect_arg(0)?,
            tags: msg.tags.clone()
        })
    }
}

parse! {
    UserNotice { tags, channel, message } => |msg: &'t Message<'t>| {
        msg.expect_command("USERNOTICE")?;
        let channel = msg.expect_arg(0)?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel,
            message: msg.data.clone(),
        })
    }
}

parse! {
    Names { name, channel, kind } => |msg: &'t Message<'t>| {
        let kind = match &*msg.command {
            "353" => {
                let users = msg.expect_data()?.split_whitespace();
                let users = users.map(Cow::Borrowed).collect();
                NamesKind::Start {
                    users
                }
            }
            "366" => {
                NamesKind::End
            }
            unknown => return Err(InvalidMessage::InvalidCommand {
                expected: "353 or 366".to_string(),
                got: unknown.to_string()
            })
        };

        let name = msg.expect_arg(0)?;
        let channel = match msg.expect_arg(1)? {
            d if d == "=" => msg.expect_arg(2)?,
            channel => channel
        };

        Ok(Self {
            name,
            channel,
            kind
        })
    }
}

parse! {
    GlobalUserState {
        user_id,
        display_name,
        color,
        emote_sets,
        badges
    } => |msg: &'t Message<'t>| {
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
            .unwrap_or_else(|| vec![Cow::from("0")]);

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

parse! {
    HostTarget { source, viewers, kind } => |msg: &'t Message<'t>| {
        msg.expect_command("HOSTTARGET")?;
        let source = msg.expect_arg(0)?;
        let (kind, viewers) = {
            let mut data = msg.expect_data()?.splitn(2,|p: char| p.is_whitespace());
            match data.next() {
                Some("-") => {
                    let viewers = data.next().and_then(|s| s.parse().ok());
                    (HostTargetKind::End, viewers)
                }
                Some(target) => {
                let target = target.into();
                let viewers = data.next().and_then(|s| s.parse().ok());
                (HostTargetKind::Start { target }, viewers)
                }
                None => return Err(InvalidMessage::ExpectedData),
            }
        };
        Ok(Self {
            source,
            kind,
            viewers,
        })
    }
}

parse! {
    Cap { capability, acknowledged } => |msg: &'t Message<'t>| {
        msg.expect_command("CAP")?;
        let acknowledged = msg.expect_arg(1)? == "ACK";
        let capability = msg.expect_data()?.clone();
        Ok(Self {
            capability,
            acknowledged,
        })
    }
}

parse! {
    ClearChat { tags, channel, name } => |msg: &'t Message<'t>| {
        msg.expect_command("CLEARCHAT")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            name: msg.expect_data().ok().cloned(),
        })
    }
}

parse! {
    ClearMsg { tags, channel, message } => |msg: &'t Message<'t>| {
        msg.expect_command("CLEARMSG")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            message: msg.expect_data().ok().cloned(),
        })
    }
}

parse! {
    IrcReady { nickname } => |msg: &'t Message<'t>| {
        msg.expect_command("001")?;
        msg.expect_arg(0).map(|nickname| Self { nickname })
    }
}

parse! {
    Join { name, channel } => |msg: &'t Message<'t>| {
        msg.expect_command("JOIN")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

parse! {
    Mode { channel, status, name,} => |msg: &'t Message<'t>| {
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

parse! {
    Notice { tags, channel, message } => |msg: &'t Message<'t>| {
        msg.expect_command("NOTICE")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            message: msg.expect_data()?.clone(),
        })
    }
}

parse! {
    Part { name, channel } => |msg: &'t Message<'t>| {
        msg.expect_command("PART")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

parse! {
    Ping { token } => |msg: &'t Message<'t>| {
        msg.expect_command("PING")?;
        msg.expect_data().map(|token| Self { token: token.clone() })
    }
}

parse! {
    Pong { token } => |msg: &'t Message<'t>| {
        msg.expect_command("PONG")?;
        msg.expect_data().map(|token| Self { token: token.clone() })
    }
}

parse! {
    Privmsg { name, channel, data, tags, } => |msg: &'t Message<'t>| {
        msg.expect_command("PRIVMSG")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
            data: msg.expect_data()?.clone(),
            tags: msg.tags.clone(),
        })
    }
}

parse! {
    Ready { username } => |msg: &'t Message<'t>| {
        msg.expect_command("376")?;
        msg.expect_arg(0).map(|username| Self { username })
    }
}

parse! {
    Reconnect => |msg: &'t Message<'t>| {
        msg.expect_command("RECONNECT").map(|_| Self{ })
    }
}

parse! {
    UserState { tags, channel } => |msg: &'t Message<'t>| {
        msg.expect_command("USERSTATE")?;
        msg.expect_arg(0).map(|channel| Self {
            channel,
            tags: msg.tags.clone(),
        })
    }
}

parse! {
    Whisper { name, data, tags, } => |msg: &'t Message<'t>| {
        msg.expect_command("WHISPER")?;
        Ok(Self {
            name: msg.expect_nick()?,
            data: msg.expect_data()?.clone(),
            tags: msg.tags.clone(),
        })
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
