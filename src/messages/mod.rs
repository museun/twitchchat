/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [Parse] on an [Message]

[subscribing]: ../client/struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../client/struct.Dispatcher.html
[Parse]: ../trait.Parse.html
[Message]: ../decode/struct.Message.html
*/

use crate::decode::Message;
use crate::Tags;

use std::borrow::Cow;

mod error;
pub use error::InvalidMessage;

mod expect;

mod parse;

/// A raw IRC message
pub type Raw<'t> = Message<'t>;

/// Acknowledgement (or not) on a CAPS request
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cap<'t> {
    /// The capability name
    pub capability: Cow<'t, str>,
    /// Whether it was acknowledged
    pub acknowledged: bool,
}

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearChat<'t> {
    /// Tags attached to the message
    pub tags: Tags<'t>,
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The user, if any, that was being purged
    pub name: Option<Cow<'t, str>>,
}

impl<'t> ClearChat<'t> {
    /// (Optional) Duration of the timeout, in seconds. If omitted, the ban is permanent.
    pub fn ban_duration(&self) -> Option<u64> {
        self.tags.get_parsed("ban-duration")
    }
}

/// When a single message has been removed from a channel.
///
/// This is triggered via /delete on IRC.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearMsg<'t> {
    /// Tags attached to the message
    pub tags: Tags<'t>,
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The message that was deleted
    pub message: Option<Cow<'t, str>>,
}

impl<'t> ClearMsg<'t> {
    /// Name of the user who sent the message
    pub fn login(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("login")
    }

    /// UUID of the message
    pub fn target_msg_id(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("target-msg-id")
    }
}

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlobalUserState<'t> {
    /// Your user-id
    pub user_id: Cow<'t, str>,
    /// Your display name, if set   
    pub display_name: Option<Cow<'t, str>>,
    /// Your color, if set. Defaults to `white`
    pub color: crate::color::Color,
    /// Your available emote sets, always contains atleast '0'
    pub emote_sets: Vec<Cow<'t, str>>,
    /// Any badges you have
    pub badges: Vec<crate::Badge<'t>>,
}

/// Event kind for determine when a Host event beings or end
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HostTargetKind<'t> {
    /// The host event started
    Start {
        /// Target channel that is being hosted
        target: Cow<'t, str>,
    },
    /// The host event ended
    End,
}

/// When a channel starts to host another channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostTarget<'t> {
    /// Source channel (the one doing the hosting).
    pub source: Cow<'t, str>,
    /// How many viewers are going along
    pub viewers: Option<usize>,
    /// What kind of event this was. e.g. `Start` or `End`
    pub kind: HostTargetKind<'t>,
}

/// Happens when the IRC connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrcReady<'t> {
    /// The name the server will refer to you as
    pub nickname: Cow<'t, str>,
}

/// User join message
///
/// The happens when a user (yourself included) joins a channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join<'t> {
    /// Name of the user that joined the channel
    pub name: Cow<'t, str>,
    /// Channel which they joined
    pub channel: Cow<'t, str>,
}

/// Status of gaining or losing moderator (operator) status
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModeStatus {
    /// Moderator status gained
    Gained,
    /// Moderator status lost
    Lost,
}

/// When a user gains or loses moderator (operator) status in a channel.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mode<'t> {
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The status. gained, or lost
    pub status: ModeStatus,
    /// The user this applies too
    pub name: Cow<'t, str>,
}

/// The kind of the Names event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NamesKind<'t> {
    /// Names begins, this'll continue until `End` is recieved
    Start {
        /// A list of user names
        users: Vec<Cow<'t, str>>,
    },
    /// Names end, this'll mark the end of the event
    End,
}

/// The names event
///
/// This'll will list people on a channel for your user
///
/// The `kind` field lets you determine if its still 'happening'
///
/// Your should keep a list of the names from the `Start` variant
///
/// And once you receive an End you'll have the complete lost
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Names<'t> {
    /// Your username
    pub name: Cow<'t, str>,
    /// The channel this event is happening for
    pub channel: Cow<'t, str>,
    /// The state of the event
    pub kind: NamesKind<'t>,
}

/// General notices from the server.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notice<'t> {
    /// The tags attached to this message
    pub tags: Tags<'t>,
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The message from the server
    pub message: Cow<'t, str>,
}

impl<'t> Notice<'t> {
    /// A message ID string. Can be used for ***i18ln***.
    ///
    /// Valid values: see [Twitch IRC: msg-id Tags](https://dev.twitch.tv/docs/irc/msg-id/).
    ///
    /// Returns None if this tag wasn't found on the message
    pub fn msg_id(&'t self) -> Option<MessageId<'t>> {
        let input = self.tags.get("msg-id")?;
        MessageId::parse(input).into()
    }
}

/// These tags apply to both the NOTICE (Twitch Commands) and NOTICE (Twitch Chat Rooms) commands.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessageId<'t> {
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
    /// Unknown message id
    Unknown(Cow<'t, str>),
}

/// User leave message
///
/// The happens when a user (yourself included) leaves a channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part<'t> {
    /// Name of the user that left the channel
    pub name: Cow<'t, str>,
    /// Channel which they left
    pub channel: Cow<'t, str>,
}

/// A ping request from the server
///
/// This is sent periodically, and handled by the `Client` internally
///
/// But you can use them however you want
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ping<'t> {
    /// Token associated with the PING event
    pub token: Cow<'t, str>,
}

/// A pong response sent from the server
///
/// This should be a response to sending a PING to the server
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pong<'t> {
    /// Token associated with the PONG event
    pub token: Cow<'t, str>,
}

/// Message sent by a user
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Privmsg<'t> {
    /// User who sent this messages
    pub name: Cow<'t, str>,
    /// Channel this message was sent on
    pub channel: Cow<'t, str>,
    /// Data that the user provided
    pub data: Cow<'t, str>,
    /// Tags attached to the message
    pub tags: Tags<'t>,
}

impl<'t> Privmsg<'t> {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber
    ///    
    pub fn badge_info(&'t self) -> Vec<crate::BadgeInfo<'t>> {
        self.tags
            .get("badge-info")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// Badges attached to this message
    ///    
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }
    /// How many bits were attached to this message
    pub fn bits(&self) -> Option<u64> {
        self.tags.get_parsed("bits")
    }

    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// display_name
    pub fn display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(s))
            .unwrap_or_default()
    }

    /// Whether the user sending this message was a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags.get_as_bool("mod")
    }

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<u64> {
        self.tags.get_parsed("room-id")
    }

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags.get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<u64> {
        self.tags.get_parsed("user-id")
    }
}

/// Happens when the Twitch connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ready<'t> {
    /// The name Twitch will refer to you as
    pub username: Cow<'t, str>,
}

/// Signals that you should reconnect and rejoin channels after a restart.
///
/// Twitch IRC processes occasionally need to be restarted. When this happens,
/// clients that have requested the IRC v3 twitch.tv/commands capability are
/// issued a RECONNECT. After a short time, the connection is closed. In this
/// case, reconnect and rejoin channels that were on the connection, as you
/// would normally.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reconnect {}

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomState<'t> {
    /// Tags attached to this message
    pub tags: Tags<'t>,
    /// Channel this event is happening on
    pub channel: Cow<'t, str>,
}

impl<'t> RoomState<'t> {
    /// Whether this room is in emote only mode
    pub fn is_emote_only(&self) -> bool {
        self.tags.get_as_bool("emote-only")
    }

    /// Whether this room is in followers only mode
    pub fn is_followers_only(&self) -> Option<FollowersOnly> {
        self.tags
            .get_parsed::<_, isize>("followers-only")
            .map(|s| match s {
                -1 => FollowersOnly::Disabled,
                0 => FollowersOnly::All,
                d => FollowersOnly::Limit(d),
            })
    }

    /// Whether this room is in r9k mode
    pub fn is_r9k(&self) -> bool {
        self.tags.get_as_bool("r9k")
    }

    /// Whether this room is in slow mode
    ///
    /// This returns the delay in which each message can be sent
    pub fn is_slow_mode(&self) -> Option<u64> {
        self.tags.get_parsed("slow").filter(|&s| s > 0)
    }

    /// Whether this room is in subs only mode
    pub fn is_subs_only(&self) -> bool {
        self.tags.get_as_bool("subs-only")
    }
}

/// The parameters for a room being in follower-only mode
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FollowersOnly {
    /// The mode is disbaled
    Disabled,
    /// All followers are allowed to speak
    All,
    /// Only those following for `n` days are allowed to speak
    Limit(isize),
}

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
            .get("badge-info")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// Badges attached to this message
    ///    
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// The user's color, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// The user's display name, if set
    pub fn display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(s))
            .unwrap_or_default()
    }

    /// A unique id (UUID) attached to this message
    ///
    /// (this is used for localization)
    pub fn id(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("id")
    }

    /// The name of the user who sent this notice
    pub fn login(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("login")
    }

    /// Whether this user is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags.get_as_bool("mod")
    }

    /// The kind of notice this message is
    pub fn msg_id(&'t self) -> Option<NoticeType<'t>> {
        let kind = self.tags.get("msg-id")?;
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
            _ => NoticeType::Unknown(kind.clone()),
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
    pub fn msg_param_display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("msg-param-displayName")
    }

    /// (Sent on only raid) The name of the source user raiding this channel.
    pub fn msg_param_login(&'t self) -> Option<&'t Cow<'t, str>> {
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
    pub fn msg_param_promo_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("msg-param-promo-name")
    }

    /// (Sent only on subgift, anonsubgift) The display name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("msg-param-recipient-display-name")
    }

    /// (Sent only on subgift, anonsubgift) The user ID of the subscription gift
    /// recipient.
    pub fn msg_param_recipient_id(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-recipient-id")
    }

    /// (Sent only on subgift, anonsubgift) The user name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_user_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("msg-param-recipient-user-name")
    }

    /// (Sent only on giftpaidupgrade) The login of the user who gifted the
    /// subscription.
    pub fn msg_param_sender_login(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("msg-param-sender-login")
    }

    /// (Sent only on giftpaidupgrade) The display name of the user who gifted
    /// the subscription.
    pub fn msg_param_sender_name(&'t self) -> Option<&'t Cow<'t, str>> {
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
    pub fn msg_param_sub_plan_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("msg-param-sub-plan-name")
    }

    /// (Sent only on raid) The number of viewers watching the source channel
    /// raiding this channel.
    pub fn msg_param_viewer_count(&self) -> Option<u64> {
        self.tags.get_parsed("msg-param-viewerCount")
    }

    /// (Sent only on ritual) The name of the ritual this notice is for. Valid
    /// value: new_chatter.
    pub fn msg_param_ritual_name(&'t self) -> Option<&'t Cow<'t, str>> {
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
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    Unknown(Cow<'t, str>),
}

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserState<'t> {
    /// Tags attached to this message
    pub tags: Tags<'t>,
    /// Channel this event happened on
    pub channel: Cow<'t, str>,
}

impl<'t> UserState<'t> {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber
    ///    
    pub fn badge_info(&'t self) -> Vec<crate::BadgeInfo<'t>> {
        self.tags
            .get("badge-info")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// Badges attached to this message
    ///    
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// The user's color, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// The user's display name, if set
    pub fn display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(s))
            .unwrap_or_default()
    }

    /// Whether this user a is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags.get_as_bool("mod")
    }
}

/// This is a collection of all possible message types
///
/// Subscribing to [events::All][all] will produce this, so you can have a single stream for multiple messages.
///
/// [all]: ../events/struct.All.html
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AllCommands<'t> {
    /// An unknown event occured
    Unknown(Raw<'t>),
    /// A capabilities event occured
    Cap(Cap<'t>),
    /// A ClearChat event occured
    ClearChat(ClearChat<'t>),
    /// A ClearMsg event occured
    ClearMsg(ClearMsg<'t>),
    /// A GlobalUserState event occured
    GlobalUserState(GlobalUserState<'t>),
    /// A HostTarget event occured
    HostTarget(HostTarget<'t>),
    /// A IrcReady event occured
    IrcReady(IrcReady<'t>),
    /// A Join event occured
    Join(Join<'t>),
    /// A Mode event occured
    Mode(Mode<'t>),
    /// A Names event occured
    Names(Names<'t>),
    /// A Notice event occured
    Notice(Notice<'t>),
    /// A Part event occured
    Part(Part<'t>),
    /// A Ping event occured
    Ping(Ping<'t>),
    /// A Pong event occured
    Pong(Pong<'t>),
    /// A Privmsg event occured
    Privmsg(Privmsg<'t>),
    /// A Ready event occured
    Ready(Ready<'t>),
    /// A Reconnect event occured
    Reconnect(Reconnect),
    /// A RoomState event occured
    RoomState(RoomState<'t>),
    /// A UserNotice event occured
    UserNotice(UserNotice<'t>),
    /// A UserState event occured
    UserState(UserState<'t>),
}

// manual impls because they are different
impl<'t> From<Raw<'t>> for AllCommands<'t> {
    fn from(msg: Raw<'t>) -> Self {
        Self::Unknown(msg)
    }
}

impl<'t> From<Reconnect> for AllCommands<'t> {
    fn from(msg: Reconnect) -> Self {
        Self::Reconnect(msg)
    }
}

macro_rules! from {
    ($($ident:tt),* $(,)?) => {
        $(
            impl<'t> From<$ident<'t>> for AllCommands<'t> {
                fn from(msg: $ident<'t>) -> Self {
                    Self::$ident(msg)
                }
            }
        )*
    };
}

// rote implementation
from! {
    Cap,
    ClearChat,
    ClearMsg,
    GlobalUserState,
    HostTarget,
    IrcReady,
    Join,
    Mode,
    Names,
    Notice,
    Part,
    Ping,
    Pong,
    Privmsg,
    Ready,
    RoomState,
    UserNotice,
    UserState,
}

#[cfg(test)]
mod tests;
