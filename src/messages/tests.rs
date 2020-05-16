use super::*;
use crate::{Parse, Tags};

#[test]
fn raw() {
    let input = ":museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :testing over here\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Raw::parse(&msg).unwrap(),
            Raw {
                raw: input.into(),
                tags: Tags::default(),
                prefix: Some(crate::decode::Prefix::User {
                    nick: "museun".into()
                }),
                command: "PRIVMSG".into(),
                args: "#museun".into(),
                data: Some("testing over here".into()),
            }
        )
    }
}

#[test]
fn user_notice_message() {
    let input = ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserNotice::parse(&msg).unwrap(),
            UserNotice {
                tags: Tags::default(),
                channel: "#museun".into(),
                message: Some("This room is no longer in slow mode.".into())
            }
        )
    }
}

#[test]
fn user_notice() {
    let input = ":tmi.twitch.tv USERNOTICE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserNotice::parse(&msg).unwrap(),
            UserNotice {
                tags: Tags::default(),
                channel: "#museun".into(),
                message: None,
            }
        )
    }
}

#[test]
fn room_state() {
    let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            RoomState::parse(&msg).unwrap(),
            RoomState {
                tags: Tags::default(),
                channel: "#museun".into()
            }
        )
    }
}

#[allow(deprecated)]
#[test]
fn names_start() {
    let input =
        ":museun!museun@museun.tmi.twitch.tv 353 museun = #museun :shaken_bot4 shaken_bot5\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Names::parse(&msg).unwrap(),
            Names {
                name: "museun".into(),
                channel: "#museun".into(),
                kind: NamesKind::Start {
                    users: vec!["shaken_bot4".into(), "shaken_bot5".into()]
                }
            }
        )
    }
}

#[allow(deprecated)]
#[test]
fn names_end() {
    let input = ":museun!museun@museun.tmi.twitch.tv 366 museun #museun :End of /NAMES list\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Names::parse(&msg).unwrap(),
            Names {
                name: "museun".into(),
                channel: "#museun".into(),
                kind: NamesKind::End
            }
        )
    }
}

#[test]
fn global_user_state() {
    let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            GlobalUserState::parse(&msg).unwrap(),
            GlobalUserState {
                user_id: "241015868".into(),
                display_name: Some("shaken_bot".into()),
                color: "#FF69B4".parse().unwrap(),
                emote_sets: vec!["0".into()],
                badges: vec![],
            }
        )
    }
}

#[test]
fn host_target() {
    let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun 1024\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            HostTarget::parse(&msg).unwrap(),
            HostTarget {
                source: "#shaken_bot".into(),
                viewers: Some(1024),
                kind: HostTargetKind::Start {
                    target: "#museun".into()
                },
            }
        )
    }
}

#[test]
fn host_target_none() {
    let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            HostTarget::parse(&msg).unwrap(),
            HostTarget {
                source: "#shaken_bot".into(),
                viewers: None,
                kind: HostTargetKind::Start {
                    target: "#museun".into()
                },
            }
        )
    }
}

#[test]
fn cap_acknowledged() {
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
        let msg = Cap::parse(&msg).unwrap();
        assert!(msg.acknowledged);
        assert_eq!(msg.capability, *expected);
    }
}

#[test]
fn cap_failed() {
    let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        let cap = Cap::parse(&msg).unwrap();
        assert!(!cap.acknowledged);
        assert_eq!(cap.capability, "foobar");
    }
}

#[test]
fn clear_chat() {
    let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearChat::parse(&msg).unwrap(),
            ClearChat {
                tags: Tags::default(),
                channel: "#museun".into(),
                name: Some("shaken_bot".into()),
            }
        )
    }
}

#[test]
fn clear_chat_empty() {
    let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearChat::parse(&msg).unwrap(),
            ClearChat {
                tags: Tags::default(),
                channel: "#museun".into(),
                name: None,
            }
        )
    }
}

#[test]
fn clear_msg() {
    let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearMsg::parse(&msg).unwrap(),
            ClearMsg {
                tags: Tags::default(),
                channel: "#museun".into(),
                message: Some("HeyGuys".into()),
            }
        )
    }
}

#[test]
fn clear_msg_empty() {
    let input = ":tmi.twitch.tv CLEARMSG #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            ClearMsg::parse(&msg).unwrap(),
            ClearMsg {
                tags: Tags::default(),
                channel: "#museun".into(),
                message: None,
            }
        )
    }
}

#[test]
fn irc_ready() {
    let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            IrcReady::parse(&msg).unwrap(),
            IrcReady {
                nickname: "shaken_bot".into()
            }
        )
    }
}

#[test]
fn join_bad_command() {
    let input = crate::decode(":tmi.twitch.tv NOT_JOIN #foo\r\n".into())
        .flatten()
        .next()
        .unwrap();

    let err = Join::parse(&input).unwrap_err();
    matches::matches!(
        err,
        InvalidMessage::InvalidCommand {..}
    );
}

#[test]
fn join_bad_nick() {
    let input = crate::decode(":tmi.twitch.tv JOIN #foo\r\n".into())
        .flatten()
        .next()
        .unwrap();

    let err = Join::parse(&input).unwrap_err();
    matches::matches!(err, InvalidMessage::ExpectedNick);
}

#[test]
fn join_bad_channel() {
    let input = crate::decode(":tmi.twitch.tv JOIN\r\n".into())
        .flatten()
        .next()
        .unwrap();

    let err = Join::parse(&input).unwrap_err();
    matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
}

#[test]
fn join() {
    let input = ":test!test@test JOIN #foo\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Join::parse(&msg).unwrap(),
            Join {
                name: "test".into(),
                channel: "#foo".into()
            }
        )
    }
}

#[allow(deprecated)]
#[test]
fn mode_lost() {
    let input = ":jtv MODE #museun -o shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Mode::parse(&msg).unwrap(),
            Mode {
                channel: "#museun".into(),
                status: ModeStatus::Lost,
                name: "shaken_bot".into()
            }
        )
    }
}

#[allow(deprecated)]
#[test]
fn mode_gained() {
    let input = ":jtv MODE #museun +o shaken_bot\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Mode::parse(&msg).unwrap(),
            Mode {
                channel: "#museun".into(),
                status: ModeStatus::Gained,
                name: "shaken_bot".into(),
            }
        )
    }
}

#[test]
fn notice() {
    let input = ":tmi.twitch.tv NOTICE #museun :This room is no longer in slow mode.\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Notice::parse(&msg).unwrap(),
            Notice {
                tags: Tags::default(),
                channel: "#museun".into(),
                message: "This room is no longer in slow mode.".into(),
            }
        )
    }
}

#[test]
fn part() {
    let input = ":test!test@test PART #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Part::parse(&msg).unwrap(),
            Part {
                name: "test".into(),
                channel: "#museun".into(),
            }
        )
    }
}

#[test]
fn ping() {
    let input = "PING :1234567890\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Ping::parse(&msg).unwrap(),
            Ping {
                token: "1234567890".into()
            }
        )
    }
}

#[test]
fn pong() {
    let input = "PONG :1234567890\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Pong::parse(&msg).unwrap(),
            Pong {
                token: "1234567890".into()
            }
        )
    }
}

#[test]
fn privmsg() {
    let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Privmsg::parse(&msg).unwrap(),
            Privmsg {
                name: "test".into(),
                channel: "#museun".into(),
                data: "this is a test".into(),
                tags: Default::default(),
            }
        )
    }
}

#[test]
fn ready() {
    let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            Ready::parse(&msg).unwrap(),
            Ready {
                username: "shaken_bot".into(),
            }
        )
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
fn user_state() {
    let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
    for msg in crate::decode(input).map(|s| s.unwrap()) {
        assert_eq!(
            UserState::parse(&msg).unwrap(),
            UserState {
                channel: "#museun".into(),
                tags: Tags::default()
            }
        )
    }
}

#[test]
fn user_notice_unknown() {
    let input = "@badge-info=subscriber/8;badges=subscriber/6,bits/100;color=#59517B;display-name=lllAirJordanlll;emotes=;flags=;id=3198b02c-eaf4-4904-9b07-eb1b2b12ba50;login=lllairjordanlll;mod=0;msg-id=resub;msg-param-cumulative-months=8;msg-param-months=0;msg-param-should-share-streak=0;msg-param-sub-plan-name=Channel\\sSubscription\\s(giantwaffle);msg-param-sub-plan=1000;room-id=22552479;subscriber=1;system-msg=lllAirJordanlll\\ssubscribed\\sat\\sTier\\s1.\\sThey\'ve\\ssubscribed\\sfor\\s8\\smonths!;tmi-sent-ts=1580932171144;user-id=44979519;user-type= :tmi.twitch.tv USERNOTICE #giantwaffle\r\n";
    let msg = crate::decode(input).next().unwrap().unwrap();
    let _msg = AllCommands::parse(&msg).unwrap();
    // eprintln!("{:#?}", msg);
}
