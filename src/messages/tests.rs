use super::*;

#[test]
fn raw_borrowed() {
    let input = ":museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :testing over here\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Raw::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Raw::<String>::parse(&msg).unwrap(),
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
fn user_notice_message_borrowed() {
    let input = ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserNotice::<&str>::parse(&msg).unwrap(),
            UserNotice {
                tags: Tags::default(),
                channel: "#museun",
                message: Some("This room is no longer in slow mode.")
            }
        )
    }
}

#[test]
fn user_notice_message_owned() {
    let input = ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserNotice::<String>::parse(&msg).unwrap(),
            UserNotice {
                tags: Tags::default(),
                channel: "#museun".to_string(),
                message: Some("This room is no longer in slow mode.".to_string())
            }
        )
    }
}

#[test]
fn user_notice_borrowed() {
    let input = ":tmi.twitch.tv USERNOTICE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserNotice::<&str>::parse(&msg).unwrap(),
            UserNotice {
                tags: Tags::default(),
                channel: "#museun",
                message: None,
            }
        )
    }
}

#[test]
fn user_notice_owned() {
    let input = ":tmi.twitch.tv USERNOTICE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserNotice::<String>::parse(&msg).unwrap(),
            UserNotice {
                tags: Tags::default(),
                channel: "#museun".to_string(),
                message: None,
            }
        )
    }
}

#[test]
fn room_state_borrowed() {
    let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            RoomState::<&str>::parse(&msg).unwrap(),
            RoomState {
                tags: Tags::default(),
                channel: "#museun"
            }
        )
    }
}

#[test]
fn room_state_owned() {
    let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            RoomState::<String>::parse(&msg).unwrap(),
            RoomState {
                tags: Tags::default(),
                channel: "#museun".to_string()
            }
        )
    }
}

#[test]
fn names_start_borrowed() {
    let input =
        ":museun!museun@museun.tmi.twitch.tv 353 museun = #museun :shaken_bot4 shaken_bot5\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Names::<&str>::parse(&msg).unwrap(),
            Names {
                name: "museun",
                channel: "#museun",
                kind: NamesKind::Start {
                    users: vec!["shaken_bot4", "shaken_bot5"]
                }
            }
        )
    }
}

#[test]
fn names_start_owned() {
    let input =
        ":museun!museun@museun.tmi.twitch.tv 353 museun = #museun :shaken_bot4 shaken_bot5\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Names::<String>::parse(&msg).unwrap(),
            Names {
                name: "museun".to_string(),
                channel: "#museun".to_string(),
                kind: NamesKind::Start {
                    users: vec!["shaken_bot4".to_string(), "shaken_bot5".to_string()]
                }
            }
        )
    }
}

#[test]
fn names_end_borrowed() {
    let input = ":museun!museun@museun.tmi.twitch.tv 366 museun #museun :End of /NAMES list\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Names::<&str>::parse(&msg).unwrap(),
            Names {
                name: "museun",
                channel: "#museun",
                kind: NamesKind::End
            }
        )
    }
}

#[test]
fn names_end_owned() {
    let input = ":museun!museun@museun.tmi.twitch.tv 366 museun #museun :End of /NAMES list\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Names::<String>::parse(&msg).unwrap(),
            Names {
                name: "museun".to_string(),
                channel: "#museun".to_string(),
                kind: NamesKind::End
            }
        )
    }
}

#[test]
fn global_user_state_borrowed() {
    let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            GlobalUserState::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            GlobalUserState::<String>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            HostTarget::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            HostTarget::<String>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            HostTarget::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            HostTarget::<String>::parse(&msg).unwrap(),
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
    for (msg, expected) in crate::decode(&input)
        .map(|s| s.unwrap())
        .zip(expected.into_iter())
    {
        let msg = Cap::<&str>::parse(&msg).unwrap();
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
    for (msg, expected) in crate::decode(&input)
        .map(|s| s.unwrap())
        .zip(expected.into_iter())
    {
        let msg = Cap::<String>::parse(&msg).unwrap();
        assert!(msg.acknowledged);
        assert_eq!(msg.capability, *expected);
    }
}

#[test]
fn cap_failed_borrowed() {
    let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        let cap = Cap::<&str>::parse(&msg).unwrap();
        assert!(!cap.acknowledged);
        assert_eq!(cap.capability, "foobar");
    }
}

#[test]
fn cap_failed_owned() {
    let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        let cap = Cap::<String>::parse(&msg).unwrap();
        assert!(!cap.acknowledged);
        assert_eq!(cap.capability, "foobar".to_string());
    }
}

#[test]
fn clear_chat_borrowed() {
    let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearChat::<&str>::parse(&msg).unwrap(),
            ClearChat {
                tags: Tags::default(),
                channel: "#museun",
                name: Some("shaken_bot"),
            }
        )
    }
}

#[test]
fn clear_chat_owned() {
    let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearChat::<String>::parse(&msg).unwrap(),
            ClearChat {
                tags: Tags::default(),
                channel: "#museun".to_string(),
                name: Some("shaken_bot".to_string()),
            }
        )
    }
}

#[test]
fn clear_chat_empty_borrowed() {
    let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearChat::<&str>::parse(&msg).unwrap(),
            ClearChat {
                tags: Tags::default(),
                channel: "#museun",
                name: None,
            }
        )
    }
}

#[test]
fn clear_chat_empty_owned() {
    let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearChat::<String>::parse(&msg).unwrap(),
            ClearChat {
                tags: Tags::default(),
                channel: "#museun".to_string(),
                name: None,
            }
        )
    }
}

#[test]
fn clear_msg_borrowed() {
    let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearMsg::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearMsg::<String>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearMsg::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearMsg::<String>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            IrcReady::<&str>::parse(&msg).unwrap(),
            IrcReady {
                nickname: "shaken_bot"
            }
        )
    }
}

#[test]
fn irc_ready_owned() {
    let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            IrcReady::<String>::parse(&msg).unwrap(),
            IrcReady {
                nickname: "shaken_bot".to_string()
            }
        )
    }
}

#[test]
fn join_bad_command_borrowed() {
    let input = crate::decode(":tmi.twitch.tv NOT_JOIN #foo\r\n")
        .flatten()
        .next()
        .unwrap();

    let err = Join::<&str>::parse(&input).unwrap_err();
    matches::matches!(
        err,
        InvalidMessage::InvalidCommand {..}
    );
}

#[test]
fn join_bad_nick_borrowed() {
    let input = crate::decode(":tmi.twitch.tv JOIN #foo\r\n")
        .flatten()
        .next()
        .unwrap();

    let err = Join::<&str>::parse(&input).unwrap_err();
    matches::matches!(err, InvalidMessage::ExpectedNick);
}

#[test]
fn join_bad_channel_borrowed() {
    let input = crate::decode(":tmi.twitch.tv JOIN\r\n")
        .flatten()
        .next()
        .unwrap();

    let err = Join::<&str>::parse(&input).unwrap_err();
    matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
}

#[test]
fn join_bad_command_owned() {
    let input = crate::decode(":tmi.twitch.tv NOT_JOIN #foo\r\n")
        .flatten()
        .next()
        .unwrap();

    let err = Join::<String>::parse(&input).unwrap_err();
    matches::matches!(
        err,
        InvalidMessage::InvalidCommand {..}
    );
}

#[test]
fn join_bad_nick_owned() {
    let input = crate::decode(":tmi.twitch.tv JOIN #foo\r\n")
        .flatten()
        .next()
        .unwrap();

    let err = Join::<String>::parse(&input).unwrap_err();
    matches::matches!(err, InvalidMessage::ExpectedNick);
}

#[test]
fn join_bad_channel_owned() {
    let input = crate::decode(":tmi.twitch.tv JOIN\r\n")
        .flatten()
        .next()
        .unwrap();

    let err = Join::<String>::parse(&input).unwrap_err();
    matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
}

#[test]
fn join_borrowed() {
    let input = ":test!test@test JOIN #foo\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Join::<&str>::parse(&msg).unwrap(),
            Join {
                name: "test",
                channel: "#foo"
            }
        )
    }
}

#[test]
fn join_owned() {
    let input = ":test!test@test JOIN #foo\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Join::<String>::parse(&msg).unwrap(),
            Join {
                name: "test".to_string(),
                channel: "#foo".to_string()
            }
        )
    }
}

#[test]
fn mode_lost_borrowed() {
    let input = ":jtv MODE #museun -o shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Mode::<&str>::parse(&msg).unwrap(),
            Mode {
                channel: "#museun",
                status: ModeStatus::Lost,
                name: "shaken_bot"
            }
        )
    }
}

#[test]
fn mode_gained_borrowed() {
    let input = ":jtv MODE #museun +o shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Mode::<&str>::parse(&msg).unwrap(),
            Mode {
                channel: "#museun",
                status: ModeStatus::Gained,
                name: "shaken_bot",
            }
        )
    }
}

#[test]
fn mode_lost_owned() {
    let input = ":jtv MODE #museun -o shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Mode::<String>::parse(&msg).unwrap(),
            Mode {
                channel: "#museun".to_string(),
                status: ModeStatus::Lost,
                name: "shaken_bot".to_string()
            }
        )
    }
}

#[test]
fn mode_gained_owned() {
    let input = ":jtv MODE #museun +o shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Mode::<String>::parse(&msg).unwrap(),
            Mode {
                channel: "#museun".to_string(),
                status: ModeStatus::Gained,
                name: "shaken_bot".to_string()
            }
        )
    }
}

#[test]
fn notice_borrowed() {
    let input = ":tmi.twitch.tv NOTICE #museun :This room is no longer in slow mode.\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Notice::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Notice::<String>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Part::<&str>::parse(&msg).unwrap(),
            Part {
                name: "test",
                channel: "#museun",
            }
        )
    }
}

#[test]
fn part_owned() {
    let input = ":test!test@test PART #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Part::<String>::parse(&msg).unwrap(),
            Part {
                name: "test".to_string(),
                channel: "#museun".to_string(),
            }
        )
    }
}

#[test]
fn ping_borrowed() {
    let input = "PING :1234567890\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Ping::<&str>::parse(&msg).unwrap(),
            Ping {
                token: "1234567890"
            }
        )
    }
}

#[test]
fn ping_owned() {
    let input = "PING :1234567890\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Ping::<String>::parse(&msg).unwrap(),
            Ping {
                token: "1234567890".to_string()
            }
        )
    }
}

#[test]
fn pong_borrowed() {
    let input = "PONG :1234567890\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Pong::<&str>::parse(&msg).unwrap(),
            Pong {
                token: "1234567890"
            }
        )
    }
}

#[test]
fn pong_owned() {
    let input = "PONG :1234567890\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Pong::<String>::parse(&msg).unwrap(),
            Pong {
                token: "1234567890".to_string()
            }
        )
    }
}

#[test]
fn privmsg_borrowed() {
    let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Privmsg::<&str>::parse(&msg).unwrap(),
            Privmsg {
                name: "test",
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Privmsg::<String>::parse(&msg).unwrap(),
            Privmsg {
                name: "test".to_string(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Ready::<&str>::parse(&msg).unwrap(),
            Ready {
                username: "shaken_bot",
            }
        )
    }
}

#[test]
fn ready_owned() {
    let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Ready::<String>::parse(&msg).unwrap(),
            Ready {
                username: "shaken_bot".to_string(),
            }
        );
    }
}

#[test]
fn reconnect() {
    let input = ":tmi.twitch.tv RECONNECT\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(Reconnect::parse(&msg).unwrap(), Reconnect {});
    }
}

#[test]
fn user_state_borrowed() {
    let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserState::<&str>::parse(&msg).unwrap(),
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
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserState::<String>::parse(&msg).unwrap(),
            UserState {
                channel: "#museun".to_string(),
                tags: Tags::default()
            }
        );
    }
}
