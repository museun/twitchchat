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
