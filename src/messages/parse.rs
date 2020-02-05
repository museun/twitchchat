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
            "HOSTARGET" => HostTarget::parse(msg)?.into(),
            "GLOBALUSERSTATE" => GlobalUserState::parse(msg)?.into(),
            "NOTICE" => Notice::parse(msg)?.into(),
            "CLEARCHAT" => ClearChat::parse(msg)?.into(),
            "CLEARMSG" => ClearMsg::parse(msg)?.into(),
            "RECONNECT" => Reconnect::parse(msg)?.into(),
            "ROOMSTATE" => RoomState::parse(msg)?.into(),
            "USERNOTICE" => UserNotice::parse(msg)?.into(),
            "USERSTATE" => UserState::parse(msg)?.into(),
            "MODE" => Mode::parse(msg)?.into(),
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
