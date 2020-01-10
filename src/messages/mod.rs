/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [TryFrom] on an [Message]

[subscribing]: ../struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../struct.Dispatcher.html
[TryFrom]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
[Message]: ../../decode/struct.Message.html
*/

use crate::decode::Message;
use crate::Tags;
use crate::{IntoOwned, StringMarker};

use std::convert::TryFrom;

/// An error returned when trying to use [TryFrom] on a [Message] to a specific [message][msg]
///
/// [TryFrom]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
/// [Message]: ../decode/struct.Message.html
/// [msg]: ./messages/index.html
#[derive(Debug)]
#[non_exhaustive]
pub enum InvalidMessage {
    /// An invalid command was found for this message
    InvalidCommand {
        /// Expected this command
        expected: String,
        /// Got this command
        got: String,
    },
    /// Expected a nickname attached to this message
    ExpectedNick,
    /// Expected an argument at a position in this message
    ExpectedArg {
        /// Argument position
        pos: usize,
    },
    /// Expected this message to have data attached
    ExpectedData,
}

// TODO (important) implement this
impl std::fmt::Display for InvalidMessage {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => unimplemented!(),
        }
    }
}

impl std::error::Error for InvalidMessage {}

pub(crate) trait Expect {
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&self) -> Result<&str, InvalidMessage>;
    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage>;
    fn expect_data(&self) -> Result<&str, InvalidMessage>;
}

impl<'a> Expect for crate::decode::Message<&'a str> {
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.command != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.command.to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<&str, InvalidMessage> {
        self.prefix
            .as_ref()
            .and_then(|s| s.nick())
            .cloned()
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage> {
        self.args
            .split_whitespace()
            .nth(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, InvalidMessage> {
        self.data.ok_or_else(|| InvalidMessage::ExpectedData)
    }
}

pub use messages::*;
pub use parse::*;

mod messages {
    use super::*;

    pub type Raw<T> = crate::decode::Message<T>;

    #[derive(Debug, Clone, PartialEq)]
    pub struct GlobalUserState<T = String>
    where
        T: StringMarker,
    {
        pub user_id: T,
        pub display_name: Option<T>,
        pub color: crate::color::Color,
        pub emote_sets: Vec<T>,
        pub badges: Vec<crate::Badge>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum HostTargetKind<T = String>
    where
        T: StringMarker,
    {
        Start { target: T },
        End,
    }

    // TODO add this case to the macro
    impl<'a> IntoOwned for HostTargetKind<&'a str> {
        type Target = HostTargetKind<String>;

        fn into_owned(&self) -> Self::Target {
            match self {
                HostTargetKind::Start { target } => HostTargetKind::Start {
                    target: target.to_string(),
                },
                HostTargetKind::End => HostTargetKind::End,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct HostTarget<T = String>
    where
        T: StringMarker,
    {
        pub source: T,
        pub viewers: Option<usize>,
        pub kind: HostTargetKind<T>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Cap<T = String>
    where
        T: StringMarker,
    {
        pub capability: T,
        pub acknowledged: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ClearChat<T = String>
    where
        T: StringMarker,
    {
        pub tags: Tags<T>,
        pub channel: T,
        pub user: Option<T>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ClearMsg<T = String>
    where
        T: StringMarker,
    {
        pub tags: Tags<T>,
        pub channel: T,
        pub message: Option<T>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct IrcReady<T = String>
    where
        T: StringMarker,
    {
        pub nickname: T,
    }

    /// User join message
    ///
    /// The happens when a user (yourself included) joins a channel
    #[derive(Debug, Clone, PartialEq)]
    pub struct Join<T = String>
    where
        T: StringMarker,
    {
        /// Name of the user that joined the channel
        pub user: T,
        /// Channel which they joined
        pub channel: T,
    }

    /// Status of gaining or losing moderator (operator) status
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum ModeStatus {
        /// Moderator status gained
        Gained,
        /// Moderator status lost
        Lost,
    }

    as_owned!(for ModeStatus);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Mode<T = String>
    where
        T: StringMarker,
    {
        pub channel: T,
        pub status: ModeStatus,
        pub user: T,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Notice<T = String>
    where
        T: StringMarker,
    {
        pub tags: Tags<T>,
        pub channel: T,
        pub message: T,
    }

    /// User leave message
    ///
    /// The happens when a user (yourself included) leaves a channel
    #[derive(Debug, Clone, PartialEq)]
    pub struct Part<T = String>
    where
        T: StringMarker,
    {
        /// Name of the user that left the channel
        pub user: T,
        /// Channel which they left
        pub channel: T,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Ping<T = String>
    where
        T: StringMarker,
    {
        /// Token associated with the PING event
        pub token: T,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Pong<T = String>
    where
        T: StringMarker,
    {
        /// Token associated with the PONG event
        pub token: T,
    }

    /// Message sent by a user
    #[derive(Debug, Clone, PartialEq)]
    pub struct Privmsg<T = String>
    where
        T: StringMarker,
    {
        /// User who sent this messages
        pub user: T,
        /// Channel this message was sent on
        pub channel: T,
        /// Data that the user provided
        pub data: T,
        /// Tags attached to the message
        pub tags: Tags<T>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Ready<T = String>
    where
        T: StringMarker,
    {
        pub username: T,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Reconnect {}

    #[derive(Debug, Clone, PartialEq)]
    pub struct UserState<T = String>
    where
        T: StringMarker,
    {
        pub tags: Tags<T>,
        pub channel: T,
    }
}

mod parse {
    use super::*;

    parse! {
        Raw {
            raw,
            tags,
            prefix,
            command,
            args,
            data
        } => |msg: &'a Message<&'a str>| {
            Ok(msg.clone())
        }
    }

    parse! {
        GlobalUserState {
            user_id,
            display_name,
            color,
            emote_sets,
            badges
        } => |msg: &'a Message<&'a str>| {
            msg.expect_command("GLOBALUSERSTATE")?;

            let user_id = msg
                .tags
                .get("user-id")
                .expect("user-id attached to message");

            let display_name = msg.tags.get("display-name");

            let color = msg
                .tags
                .get("color")
                .and_then(|s| s.parse().ok())
                .unwrap_or_default();

            let emote_sets = msg
                .tags
                .get("emotes-set")
                .map(|s| s.split(',').collect())
                .unwrap_or_else(|| vec!["0"]);

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
        HostTarget { source, viewers, kind } => |msg: &'a Message<&'a str>| {
            msg.expect_command("HOSTTARGET")?;
            let source = msg.expect_arg(0)?;
            let (kind, viewers) = if let Ok(target) = msg.expect_arg(1) {
                let viewers = msg.expect_arg(2).ok().and_then(|data| data.parse().ok());
                (HostTargetKind::Start { target }, viewers)
            } else {
                let data = msg.expect_data()?;
                if !data.starts_with("-") {
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

    parse! {
        Cap { capability, acknowledged } => |msg: &'a Message<&'a str>| {
            msg.expect_command("CAP")?;
            let acknowledged = msg.expect_arg(1)? == "ACK";
            let capability = msg.expect_data()?;
            Ok(Self {
                capability,
                acknowledged,
            })
        }
    }

    parse! {
        ClearChat { tags, channel, user } => |msg: &'a Message<&'a str>| {
            msg.expect_command("CLEARCHAT")?;
            Ok(Self {
                tags: msg.tags.clone(),
                channel: msg.expect_arg(0)?,
                user: msg.expect_data().ok(),
            })
        }
    }

    parse! {
        ClearMsg { tags, channel, message } => |msg: &'a Message<&'a str>| {
            msg.expect_command("CLEARMSG")?;
            Ok(Self {
                tags: msg.tags.clone(),
                channel: msg.expect_arg(0)?,
                message: msg.expect_data().ok(),
            })
        }
    }

    parse! {
        IrcReady { nickname } => |msg: &'a Message<&'a str>| {
            msg.expect_command("001")?;
            msg.expect_arg(0).map(|nickname| Self { nickname })
        }
    }

    parse! {
        Join { user, channel } => |msg: &'a Message<&'a str>| {
            msg.expect_command("JOIN")?;
            Ok(Self {
                user: msg.expect_nick()?,
                channel: msg.expect_arg(0)?,
            })
        }
    }

    parse! {
        Mode { channel, status, user,} => |msg: &'a Message<&'a str>| {
            msg.expect_command("MODE")?;
            let channel = msg.expect_arg(0)?;
            let status = match msg.expect_arg(1)?.chars().nth(0).unwrap() {
                '+' => ModeStatus::Gained,
                '-' => ModeStatus::Lost,
                _ => unreachable!(),
            };
            let user = msg.expect_arg(2)?;
            Ok(Self {
                channel,
                status,
                user,
            })
        }
    }

    parse! {
        Notice { tags, channel, message } => |msg: &'a Message<&'a str>| {
            msg.expect_command("NOTICE")?;
            Ok(Self {
                tags: msg.tags.clone(),
                channel: msg.expect_arg(0)?,
                message: msg.expect_data()?,
            })
        }
    }

    parse! {
        Part { user, channel } => |msg: &'a Message<&'a str>| {
            msg.expect_command("PART")?;
            Ok(Self {
                user: msg.expect_nick()?,
                channel: msg.expect_arg(0)?,
            })
        }
    }

    parse! {
        Ping { token } => |msg: &'a Message<&'a str>| {
            msg.expect_command("PING")?;
            msg.expect_data().map(|token| Self { token })
        }
    }

    parse! {
        Pong { token } => |msg: &'a Message<&'a str>| {
            msg.expect_command("PONG")?;
            msg.expect_data().map(|token| Self { token })
        }
    }

    parse! {
        Privmsg { user, channel, data, tags, } => |msg: &'a Message<&'a str>| {
            msg.expect_command("PRIVMSG")?;
            Ok(Self {
                user: msg.expect_nick()?,
                channel: msg.expect_arg(0)?,
                data: msg.expect_data()?,
                tags: msg.tags.clone(),
            })
        }
    }

    parse! {
        Ready { username } => |msg: &'a Message<&'a str>| {
            msg.expect_command("376")?;
            msg.expect_arg(0).map(|username| Self { username })
        }
    }

    parse! {
        Reconnect => |msg: &'a Message<&'a str>| {
            msg.expect_command("RECONNECT").map(|_| Self{ })
        }
    }

    parse! {
        UserState { tags, channel } => |msg: &'a Message<&'a str>| {
            msg.expect_command("USERSTATE")?;
            msg.expect_arg(0).map(|channel| Self {
                channel,
                tags: msg.tags.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto as _;

    #[test]
    fn raw_borrowed() {
        let input = ":museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :testing over here\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Raw::<&str>::try_from(&msg).unwrap(),
                Raw {
                    raw: input,
                    tags: Tags::default(),
                    prefix: Some(crate::decode::Prefix::User { nick: "museun" }),
                    command: "PRIVMSG",
                    args: "#museun",
                    data: Some("testing over here"),
                }
            )
        }
    }

    #[test]
    fn raw_owned() {
        let input = ":museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :testing over here\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Raw::<String>::try_from(&msg).unwrap(),
                Raw {
                    raw: input.to_string(),
                    tags: Tags::default(),
                    prefix: Some(crate::decode::Prefix::User {
                        nick: "museun".to_string()
                    }),
                    command: "PRIVMSG".to_string(),
                    args: "#museun".to_string(),
                    data: Some("testing over here".to_string()),
                }
            )
        }
    }

    #[test]
    fn global_user_state_borrowed() {
        let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                GlobalUserState::<&str>::try_from(&msg).unwrap(),
                GlobalUserState {
                    user_id: "241015868",
                    display_name: Some("shaken_bot"),
                    color: "#FF69B4".parse().unwrap(),
                    emote_sets: vec!["0"],
                    badges: vec![],
                }
            )
        }
    }

    #[test]
    fn global_user_state_owned() {
        let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                GlobalUserState::<String>::try_from(&msg).unwrap(),
                GlobalUserState {
                    user_id: "241015868".to_string(),
                    display_name: Some("shaken_bot".to_string()),
                    color: "#FF69B4".parse().unwrap(),
                    emote_sets: vec!["0".to_string()],
                    badges: vec![],
                }
            )
        }
    }

    #[test]
    fn host_target_borrowed() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun 1024\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<&str>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot",
                    viewers: Some(1024),
                    kind: HostTargetKind::Start { target: "#museun" },
                }
            )
        }
    }

    #[test]
    fn host_target_owned() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun 1024\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<String>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot".to_string(),
                    viewers: Some(1024),
                    kind: HostTargetKind::Start {
                        target: "#museun".to_string()
                    },
                }
            )
        }
    }

    #[test]
    fn host_target_none_borrowed() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<&str>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot",
                    viewers: None,
                    kind: HostTargetKind::Start { target: "#museun" },
                }
            )
        }
    }

    #[test]
    fn host_target_none_owned() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<String>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot".to_string(),
                    viewers: None,
                    kind: HostTargetKind::Start {
                        target: "#museun".to_string()
                    },
                }
            )
        }
    }

    #[test]
    fn cap_acknowledged_borrowed() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";
        let expected = &[
            "twitch.tv/membership",
            "twitch.tv/tags",
            "twitch.tv/commands",
        ];
        for (msg, expected) in crate::decode_many(&input)
            .map(|s| s.unwrap())
            .zip(expected.into_iter())
        {
            let msg: Cap<&str> = (&msg).try_into().unwrap();
            assert!(msg.acknowledged);
            assert_eq!(msg.capability, *expected);
        }
    }

    #[test]
    fn cap_acknowledged_owned() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";
        let expected = &[
            "twitch.tv/membership",
            "twitch.tv/tags",
            "twitch.tv/commands",
        ];
        for (msg, expected) in crate::decode_many(&input)
            .map(|s| s.unwrap())
            .zip(expected.into_iter())
        {
            let msg: Cap<String> = (&msg).try_into().unwrap();
            assert!(msg.acknowledged);
            assert_eq!(msg.capability, *expected);
        }
    }

    #[test]
    fn cap_failed_borrowed() {
        let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            let cap = Cap::<&str>::try_from(&msg).unwrap();
            assert!(!cap.acknowledged);
            assert_eq!(cap.capability, "foobar");
        }
    }

    #[test]
    fn cap_failed_owned() {
        let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            let cap = Cap::<String>::try_from(&msg).unwrap();
            assert!(!cap.acknowledged);
            assert_eq!(cap.capability, "foobar".to_string());
        }
    }

    #[test]
    fn clear_chat_borrowed() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearChat::<&str>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun",
                    user: Some("shaken_bot"),
                }
            )
        }
    }

    #[test]
    fn clear_chat_owned() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearChat::<String>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun".to_string(),
                    user: Some("shaken_bot".to_string()),
                }
            )
        }
    }

    #[test]
    fn clear_chat_empty_borrowed() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearChat::<&str>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun",
                    user: None,
                }
            )
        }
    }

    #[test]
    fn clear_chat_empty_owned() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearChat::<String>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun".to_string(),
                    user: None,
                }
            )
        }
    }

    #[test]
    fn clear_msg_borrowed() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearMsg::<&str>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun",
                    message: Some("HeyGuys"),
                }
            )
        }
    }

    #[test]
    fn clear_msg_owned() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearMsg::<String>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun".to_string(),
                    message: Some("HeyGuys".to_string()),
                }
            )
        }
    }

    #[test]
    fn clear_msg_empty_borrowed() {
        let input = ":tmi.twitch.tv CLEARMSG #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearMsg::<&str>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun",
                    message: None,
                }
            )
        }
    }

    #[test]
    fn clear_msg_empty_owned() {
        let input = ":tmi.twitch.tv CLEARMSG #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearMsg::<String>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun".to_string(),
                    message: None,
                }
            )
        }
    }

    #[test]
    fn irc_ready_borrowed() {
        let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                IrcReady::<&str>::try_from(&msg).unwrap(),
                IrcReady {
                    nickname: "shaken_bot"
                }
            )
        }
    }

    #[test]
    fn irc_ready_owned() {
        let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                IrcReady::<String>::try_from(&msg).unwrap(),
                IrcReady {
                    nickname: "shaken_bot".to_string()
                }
            )
        }
    }

    #[test]
    fn join_bad_command_borrowed() {
        let input = crate::decode_many(":tmi.twitch.tv NOT_JOIN #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<&str>::try_from(&input).unwrap_err();
        matches::matches!(
            err,
            InvalidMessage::InvalidCommand {..}
        );
    }

    #[test]
    fn join_bad_nick_borrowed() {
        let input = crate::decode_many(":tmi.twitch.tv JOIN #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<&str>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedNick);
    }

    #[test]
    fn join_bad_channel_borrowed() {
        let input = crate::decode_many(":tmi.twitch.tv JOIN\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<&str>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
    }

    #[test]
    fn join_bad_command_owned() {
        let input = crate::decode_many(":tmi.twitch.tv NOT_JOIN #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<String>::try_from(&input).unwrap_err();
        matches::matches!(
            err,
            InvalidMessage::InvalidCommand {..}
        );
    }

    #[test]
    fn join_bad_nick_owned() {
        let input = crate::decode_many(":tmi.twitch.tv JOIN #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<String>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedNick);
    }

    #[test]
    fn join_bad_channel_owned() {
        let input = crate::decode_many(":tmi.twitch.tv JOIN\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<String>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
    }

    #[test]
    fn join_borrowed() {
        let input = ":test!test@test JOIN #foo\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Join::<&str>::try_from(&msg).unwrap(),
                Join {
                    user: "test",
                    channel: "#foo"
                }
            )
        }
    }

    #[test]
    fn join_owned() {
        let input = ":test!test@test JOIN #foo\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Join::<String>::try_from(&msg).unwrap(),
                Join {
                    user: "test".to_string(),
                    channel: "#foo".to_string()
                }
            )
        }
    }

    #[test]
    fn mode_lost_borrowed() {
        let input = ":jtv MODE #museun -o shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Mode::<&str>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun",
                    status: ModeStatus::Lost,
                    user: "shaken_bot"
                }
            )
        }
    }

    #[test]
    fn mode_gained_borrowed() {
        let input = ":jtv MODE #museun +o shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Mode::<&str>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun",
                    status: ModeStatus::Gained,
                    user: "shaken_bot",
                }
            )
        }
    }

    #[test]
    fn mode_lost_owned() {
        let input = ":jtv MODE #museun -o shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Mode::<String>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun".to_string(),
                    status: ModeStatus::Lost,
                    user: "shaken_bot".to_string()
                }
            )
        }
    }

    #[test]
    fn mode_gained_owned() {
        let input = ":jtv MODE #museun +o shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Mode::<String>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun".to_string(),
                    status: ModeStatus::Gained,
                    user: "shaken_bot".to_string()
                }
            )
        }
    }

    #[test]
    fn notice_borrowed() {
        let input = ":tmi.twitch.tv NOTICE #museun :This room is no longer in slow mode.\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Notice::<&str>::try_from(&msg).unwrap(),
                Notice {
                    tags: Tags::default(),
                    channel: "#museun",
                    message: "This room is no longer in slow mode.",
                }
            )
        }
    }

    #[test]
    fn notice_owned() {
        let input = ":tmi.twitch.tv NOTICE #museun :This room is no longer in slow mode.\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Notice::<String>::try_from(&msg).unwrap(),
                Notice {
                    tags: Tags::default(),
                    channel: "#museun".to_string(),
                    message: "This room is no longer in slow mode.".to_string(),
                }
            )
        }
    }

    #[test]
    fn part_borrowed() {
        let input = ":test!test@test PART #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Part::<&str>::try_from(&msg).unwrap(),
                Part {
                    user: "test",
                    channel: "#museun",
                }
            )
        }
    }

    #[test]
    fn part_owned() {
        let input = ":test!test@test PART #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Part::<String>::try_from(&msg).unwrap(),
                Part {
                    user: "test".to_string(),
                    channel: "#museun".to_string(),
                }
            )
        }
    }

    #[test]
    fn ping_borrowed() {
        let input = "PING :1234567890\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Ping::<&str>::try_from(&msg).unwrap(),
                Ping {
                    token: "1234567890"
                }
            )
        }
    }

    #[test]
    fn ping_owned() {
        let input = "PING :1234567890\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Ping::<String>::try_from(&msg).unwrap(),
                Ping {
                    token: "1234567890".to_string()
                }
            )
        }
    }

    #[test]
    fn pong_borrowed() {
        let input = "PONG :1234567890\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Pong::<&str>::try_from(&msg).unwrap(),
                Pong {
                    token: "1234567890"
                }
            )
        }
    }

    #[test]
    fn pong_owned() {
        let input = "PONG :1234567890\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Pong::<String>::try_from(&msg).unwrap(),
                Pong {
                    token: "1234567890".to_string()
                }
            )
        }
    }

    #[test]
    fn privmsg_borrowed() {
        let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Privmsg::<&str>::try_from(&msg).unwrap(),
                Privmsg {
                    user: "test",
                    channel: "#museun",
                    data: "this is a test",
                    tags: Default::default(),
                }
            )
        }
    }

    #[test]
    fn privmsg_owned() {
        let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Privmsg::<String>::try_from(&msg).unwrap(),
                Privmsg {
                    user: "test".to_string(),
                    channel: "#museun".to_string(),
                    data: "this is a test".to_string(),
                    tags: Default::default(),
                }
            )
        }
    }

    #[test]
    fn ready_borrowed() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Ready::<&str>::try_from(&msg).unwrap(),
                Ready {
                    username: "shaken_bot",
                }
            )
        }
    }

    #[test]
    fn ready_owned() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Ready::<String>::try_from(&msg).unwrap(),
                Ready {
                    username: "shaken_bot".to_string(),
                }
            );
        }
    }

    #[test]
    fn reconnect() {
        let input = ":tmi.twitch.tv RECONNECT\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(Reconnect::try_from(&msg).unwrap(), Reconnect {});
        }
    }

    #[test]
    fn user_state_borrowed() {
        let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                UserState::<&str>::try_from(&msg).unwrap(),
                UserState {
                    channel: "#museun",
                    tags: Tags::default()
                }
            )
        }
    }

    #[test]
    fn user_state_owned() {
        let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                UserState::<String>::try_from(&msg).unwrap(),
                UserState {
                    channel: "#museun".to_string(),
                    tags: Tags::default()
                }
            );
        }
    }
}

// export_modules! {
//     cap
//     clear_chat
//     clear_msg
//     global_user_state
//     host_target
//     irc_ready
//     join
//     mode
//     notice
//     part
//     ping
//     pong
//     priv_msg
//     raw
//     ready
//     reconnect
//     user_state
// }
