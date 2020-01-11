use super::*;

parse! {
    bare Raw {
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
    RoomState { tags, channel } => |msg: &'a Message<&'a str>| {
        msg.expect_command("ROOMSTATE")?;
        Ok(Self {
            channel: msg.expect_arg(0)?,
            tags: msg.tags.clone()
        })
    }
}

parse! {
    UserNotice { tags, channel, message } => |msg: &'a Message<&'a str>| {
        msg.expect_command("USERNOTICE")?;
        let channel = msg.expect_arg(0)?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel,
            message: msg.data,
        })
    }
}

parse! {
    Names { user, channel, kind } => |msg: &'a Message<&'a str>| {
        let kind = match msg.command {
            "353" => {
                NamesKind::Start {
                    users: msg.expect_data()?.split_whitespace().collect()
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

        let user = msg.expect_arg(0)?;
        let channel = match msg.expect_arg(1)? {
            "=" => msg.expect_arg(2)?,
            channel => channel
        };

        Ok(Self {
            user,
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
