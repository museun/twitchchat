use super::*;

/// General notices from the server.
#[derive(Debug, PartialEq, Clone)]
pub struct Notice {
    pub tags: Tags,
    /// The channel this event happened on
    pub channel: String,
    /// The message from the server
    pub message: String,
}

impl Notice {
    /// A message ID string. Can be used for i18ln. Valid values: see Twitch IRC: [msg-id Tags](https://dev.twitch.tv/docs/irc/msg-id/).
    pub fn msg_id(&self) -> MessageId {
        self.get("msg-id")
            .map(MessageId::parse)
            .expect("valid msg-id")
    }
}

impl Tag for Notice {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}

/// These tags apply to both the NOTICE (Twitch Commands) and NOTICE (Twitch Chat Rooms) commands.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum MessageId {
    /// <user> is already banned in this channel.
    AlreadyBanned,
    /// This room is not in emote-only mode.
    AlreadyEmoteOnlyOff,
    /// This room is already in emote-only mode.
    AlreadyEmoteOnlyOn,
    /// This room is not in r9k mode.
    AlreadyR9kOff,
    /// This room is already in r9k mode.
    AlreadyR9kOn,
    /// This room is not in subscribers-only mode.
    AlreadySubsOff,
    /// This room is already in subscribers-only mode.
    AlreadySubsOn,
    /// You cannot ban admin <user>. Please email support@twitch.tv if an admin
    /// is being abusive.
    BadBanAdmin,
    /// You cannot ban anonymous users.
    BadBanAnon,
    /// You cannot ban the broadcaster.
    BadBanBroadcaster,
    /// You cannot ban global moderator <user>. Please email support@twitch.tv
    /// if a global moderator is being abusive.
    BadBanGlobalMod,
    /// You cannot ban moderator <user> unless you are the owner of this
    /// channel.
    BadBanMod,
    /// You cannot ban yourself.
    BadBanSelf,
    /// You cannot ban staff <user>. Please email support@twitch.tv if a staff
    /// member is being abusive.
    BadBanStaff,
    /// Failed to start commercial.
    BadCommercialError,
    /// You cannot delete the broadcaster's messages.
    BadDeleteMessageBroadcaster,
    /// You cannot delete messages from another moderator <user>.
    BadDeleteMessageMod,
    /// There was a problem hosting <channel>. Please try again in a minute.
    BadHostError,
    /// This channel is already hosting <channel>.
    BadHostHosting,
    /// Host target cannot be changed more than <number> times every half hour.
    BadHostRateExceeded,
    /// This channel is unable to be hosted.
    BadHostRejected,
    /// A channel cannot host itself.
    BadHostSelf,
    /// Sorry, /marker is not available through this client.
    BadMarkerClient,
    /// <user> is banned in this channel. You must unban this user before
    /// granting mod status.
    BadModBanned,
    /// <user> is already a moderator of this channel.
    BadModMod,
    /// You cannot set slow delay to more than <number> seconds.
    BadSlowDuration,
    /// You cannot timeout admin <user>. Please email support@twitch.tv if an
    /// admin is being abusive.
    BadTimeoutAdmin,
    /// You cannot timeout anonymous users.
    BadTimeoutAnon,
    /// You cannot timeout the broadcaster.
    BadTimeoutBroadcaster,
    /// You cannot time a user out for more than <seconds>.
    BadTimeoutDuration,
    /// You cannot timeout global moderator <user>. Please email
    /// support@twitch.tv if a global moderator is being abusive.
    BadTimeoutGlobalMod,
    /// You cannot timeout moderator <user> unless you are the owner of this
    /// channel.
    BadTimeoutMod,
    /// You cannot timeout yourself.
    BadTimeoutSelf,
    /// You cannot timeout staff <user>. Please email support@twitch.tv if a
    /// staff member is being abusive.
    BadTimeoutStaff,
    /// <user> is not banned from this channel.
    BadUnbanNoBan,
    /// There was a problem exiting host mode. Please try again in a minute.
    BadUnhostError,
    /// <user> is not a moderator of this channel.
    BadUnmodMod,
    /// <user> is now banned from this channel.
    BanSuccess,
    /// Commands available to you in this room (use /help <command> for
    /// details): <list of commands>
    CmdsAvailable,
    /// Your color has been changed.
    ColorChanged,
    /// Initiating <number> second commercial break. Please keep in mind that
    /// your stream is still live and not everyone will get a commercial.
    CommercialSuccess,
    /// The message from <user> is now deleted.
    DeleteMessageSuccess,
    /// This room is no longer in emote-only mode.
    EmoteOnlyOff,
    /// This room is now in emote-only mode.
    EmoteOnlyOn,
    /// This room is no longer in followers-only mode. (Note: These three
    /// “followers” tags are broadcast to a channel, when a moderator makes
    /// changes.)
    FollowersOff,
    /// This room is now in <duration> followers-only mode. (Examples: “This
    /// room is now in 2 week followers-only mode.” or “This room is now in 1
    /// minute followers-only mode.”)followers_onzero
    FollowersOn,
    /// This room is now in followers-only mode.
    FollowersOnZero,
    /// Exited host mode.
    HostOff,
    /// Now hosting <channel>.
    HostOn,
    /// <user> is now hosting you.
    HostSuccess,
    /// <user> is now hosting you for up to <number> viewers.
    HostSuccessViewers,
    /// <channel> has gone offline. Exiting host mode.
    HostTargetWentOffline,
    /// <number> host commands remaining this half hour.
    HostsRemaining,
    /// Invalid username: <user>
    InvalidUser,
    /// You have added <user> as a moderator of this channel.
    ModSuccess,
    /// You are permanently banned from talking in <channel>.
    MsgBanned,
    /// Your message was not sent because it contained too many unprocessable
    /// characters. If you believe this is an error, please rephrase and try
    /// again.
    MsgBadCharacters,
    /// Your message was not sent because your account is not in good standing
    /// in this channel.
    MsgChannelBlocked,
    /// This channel has been suspended.
    MsgChannelSuspended,
    /// Your message was not sent because it is identical to the previous one
    /// you sent, less than 30 seconds ago.
    MsgDuplicate,
    /// This room is in emote only mode. You can find your currently available
    /// emoticons using the smiley in the chat text area.
    MsgEmoteonly,
    /// You must Facebook Connect to send messages to this channel. You can
    /// Facebook Connect in your Twitch settings under the connections tab.
    MsgFacebook,
    /// This room is in <duration> followers-only mode. Follow <channel> to join
    /// the community! (Note: These 3 “msg_followers” tags are kickbacks to a
    /// user who does not meet the criteria; i.e., does not follow or has not
    /// followed long enough.)
    MsgFollowersonly,
    /// This room is in <duration1> followers-only mode. You have been following
    /// for <duration2>. Continue following to chat!
    MsgFollowersonlyFollowed,
    /// This room is in followers-only mode. Follow <channel> to join the
    /// community!
    MsgFollowersonlyZero,
    /// This room is in r9k mode and the message you attempted to send is not
    /// unique.
    MsgR9k,
    /// Your message was not sent because you are sending messages too quickly.
    MsgRatelimit,
    /// Hey! Your message is being checked by mods and has not been sent.
    MsgRejected,
    /// Your message wasn't posted due to conflicts with the channel's
    /// moderation settings.
    MsgRejectedMandatory,
    /// The room was not found.
    MsgRoomNotFound,
    /// This room is in slow mode and you are sending messages too quickly. You
    /// will be able to talk again in <number> seconds.
    MsgSlowmode,
    /// This room is in subscribers only mode. To talk, purchase a channel
    /// subscription at https://www.twitch.tv/products/<broadcaster login
    /// name>/ticket?ref=subscriber_only_mode_chat.
    MsgSubsonly,
    /// Your account has been suspended.
    MsgSuspended,
    /// You are banned from talking in <channel> for <number> more seconds.
    MsgTimedout,
    /// This room requires a verified email address to chat. Please verify your
    /// email at https://www.twitch.tv/settings/profile.
    MsgVerifiedEmail,
    /// No help available.
    NoHelp,
    /// There are no moderators of this channel.
    NoMods,
    /// No channel is currently being hosted.
    NotHosting,
    /// You don’t have permission to perform that action.
    NoPermission,
    /// This room is no longer in r9k mode.
    R9kOff,
    /// This room is now in r9k mode.
    R9kOn,
    /// You already have a raid in progress.
    RaidErrorAlreadyRaiding,
    /// You cannot raid this channel.
    RaidErrorForbidden,
    /// A channel cannot raid itself.
    RaidErrorSelf,
    /// Sorry, you have more viewers than the maximum currently supported by
    /// raids right now.
    RaidErrorTooManyViewers,
    /// There was a problem raiding <channel>. Please try again in a minute.
    RaidErrorUnexpected,
    /// This channel is intended for mature audiences.
    RaidNoticeMature,
    /// This channel has follower or subscriber only chat.
    RaidNoticeRestrictedChat,
    /// The moderators of this channel are: <list of users>
    RoomMods,
    /// This room is no longer in slow mode.
    SlowOff,
    /// This room is now in slow mode. You may send messages every <number>
    /// seconds.
    SlowOn,
    /// This room is no longer in subscribers-only mode.
    SubsOff,
    /// This room is now in subscribers-only mode.
    SubsOn,
    /// <user> is not timed out from this channel.
    TimeoutNoTimeout,
    /// <user> has been timed out for <duration> seconds.
    TimeoutSuccess,
    /// The community has closed channel <channel> due to Terms of Service
    /// violations.
    TosBan,
    /// Only turbo users can specify an arbitrary hex color. Use one of the
    /// following instead: <list of colors>.
    TurboOnlyColor,
    /// <user> is no longer banned from this channel.
    UnbanSuccess,
    /// You have removed <user> as a moderator of this channel.
    UnmodSuccess,
    /// You do not have an active raid.
    UnraidErrorNoActiveRaid,
    /// There was a problem stopping the raid. Please try again in a minute.
    UnraidErrorUnexpected,
    /// The raid has been cancelled.
    UnraidSuccess,
    /// Unrecognized command: <command>
    UnrecognizedCmd,
    /// The command <command> cannot be used in a chatroom.
    UnsupportedChatroomsCmd,
    /// <user> is permanently banned. Use "/unban" to remove a ban.
    UntimeoutBanned,
    /// <user> is no longer timed out in this channel.
    UntimeoutSuccess,
    /// Usage: “/ban <username> [reason]” - Permanently prevent a user from
    /// chatting. Reason is optional and will be shown to the target and other
    /// moderators. Use “/unban” to remove a ban.
    UsageBan,
    /// Usage: “/clear” - Clear chat history for all users in this room.
    UsageClear,
    /// Usage: “/color” <color> - Change your username color. Color must be in
    /// hex (#000000) or one of the following: Blue, BlueViolet, CadetBlue,
    /// Chocolate, Coral, DodgerBlue, Firebrick, GoldenRod, Green, HotPink,
    /// OrangeRed, Red, SeaGreen, SpringGreen, YellowGreen.
    UsageColor,
    /// Usage: “/commercial [length]” - Triggers a commercial. Length (optional)
    /// must be a positive number of seconds.
    UsageCommercial,
    /// Usage: “/disconnect” - Reconnects to chat.
    UsageDisconnect,
    /// Usage: /emoteonlyoff” - Disables emote-only mode.
    UsageEmoteOnlyOff,
    /// Usage: “/emoteonly” - Enables emote-only mode (only emoticons may be
    /// used in chat). Use /emoteonlyoff to disable.
    UsageEmoteOnlyOn,
    /// Usage: /followersoff” - Disables followers-only mode.
    UsageFollowersOff,
    /// Usage: “/followers - Enables followers-only mode (only users who have
    /// followed for “duration” may chat). Examples: “30m”, “1 week”, “5 days 12
    /// hours”. Must be less than 3 months.
    UsageFollowersOn,
    /// Usage: “/help” - Lists the commands available to you in this room.
    UsageHelp,
    /// Usage: “/host <channel>” - Host another channel. Use “/unhost” to unset
    /// host mode.
    UsageHost,
    /// Usage: “/marker <optional comment>” - Adds a stream marker (with an
    /// optional comment, max 140 characters) at the current timestamp. You can
    /// use markers in the Highlighter for easier editing.
    UsageMarker,
    /// Usage: “/me <message>” - Send an “emote” message in the third person.
    UsageMe,
    /// Usage: “/mod <username>” - Grant mod status to a user. Use “/mods” to
    /// list the moderators of this channel.
    UsageMod,
    /// Usage: “/mods” - Lists the moderators of this channel.
    UsageMods,
    /// Usage: “/r9kbetaoff” - Disables r9k mode.
    UsageR9kOff,
    /// Usage: “/r9kbeta” - Enables r9k mode. Use “/r9kbetaoff“ to disable.
    UsageR9kOn,
    /// Usage: “/raid <channel>” - Raid another channel. Use “/unraid” to cancel
    /// the Raid.
    UsageRaid,
    /// Usage: “/slowoff” - Disables slow mode.
    UsageSlowOff,
    /// Usage: “/slow” [duration] - Enables slow mode (limit how often users may
    /// send messages). Duration (optional, default=<number>) must be a positive
    /// integer number of seconds. Use “/slowoff” to disable.
    UsageSlowOn,
    /// Usage: “/subscribersoff” - Disables subscribers-only mode.
    UsageSubsOff,
    /// Usage: “/subscribers” - Enables subscribers-only mode (only subscribers
    /// may chat in this channel). Use “/subscribersoff” to disable.
    UsageSubsOn,
    /// Usage: “/timeout <username> [duration][time unit] [reason]" -
    /// Temporarily prevent a user from chatting. Duration (optional, default=10
    /// minutes) must be a positive integer; time unit (optional, default=s)
    /// must be one of s, m, h, d, w; maximum duration is 2 weeks. Combinations
    /// like 1d2h are also allowed. Reason is optional and will be shown to the
    /// target user and other moderators. Use “untimeout” to remove a timeout.
    UsageTimeout,
    /// Usage: “/unban <username>” - Removes a ban on a user.
    UsageUnban,
    /// Usage: “/unhost” - Stop hosting another channel.
    UsageUnhost,
    /// Usage: “/unmod <username>” - Revoke mod status from a user. Use “/mods”
    /// to list the moderators of this channel.
    UsageUnmod,
    /// Usage: “/unraid” - Cancel the Raid.
    UsageUnraid,
    /// Usage: “/raid <username>” - Removes a timeout on a user.
    UsageUntimeout,
    /// You have been banned from sending whispers.
    WhisperBanned,
    /// That user has been banned from receiving whispers.
    WhisperBannedRecipient,
    /// Usage: <login> <message>
    WhisperInvalidArgs,
    /// No user matching that login.
    WhisperInvalidLogin,
    /// You cannot whisper to yourself.
    WhisperInvalidSelf,
    /// You are sending whispers too fast. Try again in a minute.
    WhisperLimitPerMin,
    /// You are sending whispers too fast. Try again in a second.
    WhisperLimitPerSec,
    /// Your settings prevent you from sending this whisper.
    WhisperRestricted,
    /// That user's settings prevent them from receiving this whisper.
    WhisperRestrictedRecipient,
}

impl MessageId {
    fn parse(s: &str) -> Self {
        use MessageId::*;
        match s {
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
            _ => unreachable!(),
        }
    }
}
