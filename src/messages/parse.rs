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

impl<'a> Parse<&'a Message<&'a str>> for AllCommands<&'a str> {
    fn parse(msg: &'a Message<&'a str>) -> Result<Self, InvalidMessage> {
        let out = match msg.command {
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
            "USERSTATE" => UserState::parse(msg)?.into(),
            "MODE" => Mode::parse(msg)?.into(),
            _ => msg.clone().into(),
        };
        Ok(out)
    }
}

impl<'a> Parse<&'a Message<&'a str>> for AllCommands<String> {
    fn parse(msg: &'a Message<&'a str>) -> Result<Self, InvalidMessage> {
        AllCommands::<&'a str>::parse(msg).map(|ok| ok.as_owned())
    }
}

impl<'a, T> Conversion<'a> for AllCommands<T>
where
    T: StringMarker + Conversion<'a>,
    <T as Conversion<'a>>::Borrowed: StringMarker,
    <T as Conversion<'a>>::Owned: StringMarker,
{
    type Owned = AllCommands<T::Owned>;
    type Borrowed = AllCommands<T::Borrowed>;

    fn as_borrowed(&'a self) -> Self::Borrowed {
        use AllCommands::*;
        match self {
            Unknown(msg) => Unknown(msg.as_borrowed()),
            Cap(msg) => Cap(msg.as_borrowed()),
            ClearChat(msg) => ClearChat(msg.as_borrowed()),
            ClearMsg(msg) => ClearMsg(msg.as_borrowed()),
            GlobalUserState(msg) => GlobalUserState(msg.as_borrowed()),
            HostTarget(msg) => HostTarget(msg.as_borrowed()),
            IrcReady(msg) => IrcReady(msg.as_borrowed()),
            Join(msg) => Join(msg.as_borrowed()),
            Mode(msg) => Mode(msg.as_borrowed()),
            Names(msg) => Names(msg.as_borrowed()),
            Notice(msg) => Notice(msg.as_borrowed()),
            Part(msg) => Part(msg.as_borrowed()),
            Ping(msg) => Ping(msg.as_borrowed()),
            Pong(msg) => Pong(msg.as_borrowed()),
            Privmsg(msg) => Privmsg(msg.as_borrowed()),
            Ready(msg) => Ready(msg.as_borrowed()),
            Reconnect(msg) => Reconnect(msg.as_borrowed()),
            RoomState(msg) => RoomState(msg.as_borrowed()),
            UserNotice(msg) => UserNotice(msg.as_borrowed()),
            UserState(msg) => UserState(msg.as_borrowed()),
        }
    }

    fn as_owned(&self) -> Self::Owned {
        use AllCommands::*;
        match self {
            Unknown(msg) => Unknown(msg.as_owned()),
            Cap(msg) => Cap(msg.as_owned()),
            ClearChat(msg) => ClearChat(msg.as_owned()),
            ClearMsg(msg) => ClearMsg(msg.as_owned()),
            GlobalUserState(msg) => GlobalUserState(msg.as_owned()),
            HostTarget(msg) => HostTarget(msg.as_owned()),
            IrcReady(msg) => IrcReady(msg.as_owned()),
            Join(msg) => Join(msg.as_owned()),
            Mode(msg) => Mode(msg.as_owned()),
            Names(msg) => Names(msg.as_owned()),
            Notice(msg) => Notice(msg.as_owned()),
            Part(msg) => Part(msg.as_owned()),
            Ping(msg) => Ping(msg.as_owned()),
            Pong(msg) => Pong(msg.as_owned()),
            Privmsg(msg) => Privmsg(msg.as_owned()),
            Ready(msg) => Ready(msg.as_owned()),
            Reconnect(msg) => Reconnect(msg.as_owned()),
            RoomState(msg) => RoomState(msg.as_owned()),
            UserNotice(msg) => UserNotice(msg.as_owned()),
            UserState(msg) => UserState(msg.as_owned()),
        }
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
    Names { name, channel, kind } => |msg: &'a Message<&'a str>| {
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

        let name = msg.expect_arg(0)?;
        let channel = match msg.expect_arg(1)? {
            "=" => msg.expect_arg(2)?,
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
    } => |msg: &'a Message<&'a str>| {
        msg.expect_command("GLOBALUSERSTATE")?;

        let user_id = msg
            .tags
            .get("user-id")
            .expect("user-id attached to message");

        let display_name = msg.tags.get("display-name").cloned();

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
    ClearChat { tags, channel, name } => |msg: &'a Message<&'a str>| {
        msg.expect_command("CLEARCHAT")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            name: msg.expect_data().ok(),
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
    Join { name, channel } => |msg: &'a Message<&'a str>| {
        msg.expect_command("JOIN")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

parse! {
    Mode { channel, status, name,} => |msg: &'a Message<&'a str>| {
        msg.expect_command("MODE")?;
        let channel = msg.expect_arg(0)?;
        let status = match msg.expect_arg(1)?.chars().nth(0).unwrap() {
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
    Part { name, channel } => |msg: &'a Message<&'a str>| {
        msg.expect_command("PART")?;
        Ok(Self {
            name: msg.expect_nick()?,
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
    Privmsg { name, channel, data, tags, } => |msg: &'a Message<&'a str>| {
        msg.expect_command("PRIVMSG")?;
        Ok(Self {
            name: msg.expect_nick()?,
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
