use super::*;

pub enum AllCommands<'a> {
    Raw(IrcMessage<'a>),
    // IrcReady(IrcReady<'a>),
    // Ready(Ready<'a>),
    // Cap(Cap<'a>),
    // ClearChat(ClearChat<'a>),
    // ClearMsg(ClearMsg<'a>),
    // GlobalUserState(GlobalUserState<'a>),
    // HostTarget(HostTarget<'a>),
    // Join(Join<'a>),
    // Notice(Notice<'a>),
    // Part(Part<'a>),
    // Ping(Ping<'a>),
    // Pong(Pong<'a>),
    // Privmsg(Privmsg<'a>),
    // Reconnect(Reconnect<'a>),
    // RoomState(RoomState<'a>),
    // UserNotice(UserNotice<'a>),
    // UserState(UserState<'a>),
    // Whisper(Whisper<'a>),
}

impl<'a> FromIrcMessage<'a> for AllCommands<'a> {
    type Error = Infallible;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        let this = match msg.get_command() {
            // "001" => Self::IrcReady(IrcReady::from_irc(msg)?),
            // "376" => Self::Ready(Ready::from_irc(msg)?),
            // "CAP" => Self::Cap(Cap::from_irc(msg)?),
            // "CLEARCHAT" => Self::ClearChat(ClearChat::from_irc(msg)?),
            // "CLEARMSG" => Self::ClearMsg(ClearMsg::from_irc(msg)?),
            // "GLOBALUSERSTATE" => Self::GlobalUserState(GlobalUserState::from_irc(msg)?),
            // "HOSTARGET" => Self::HostTarget(HostTarget::from_irc(msg)?),
            // "JOIN" => Self::Join(Join::from_irc(msg)?),
            // "NOTICE" => Self::Notice(Notice::from_irc(msg)?),
            // "PART" => Self::Part(Part::from_irc(msg)?),
            // "PING" => Self::Ping(Ping::from_irc(msg)?),
            // "PONG" => Self::Pong(Pong::from_irc(msg)?),
            // "PRIVMSG" => Self::Privmsg(Privmsg::from_irc(msg)?),
            // "RECONNECT" => Self::Reconnect(Reconnect::from_irc(msg)?),
            // "ROOMSTATE" => Self::RoomState(RoomState::from_irc(msg)?),
            // "USERNOTICE" => Self::UserNotice(UserNotice::from_irc(msg)?),
            // "USERSTATE" => Self::UserState(UserState::from_irc(msg)?),
            // "WHISPER" => Self::Whisper(Whisper::from_irc(msg)?),
            _ => Self::Raw(IrcMessage::from_irc(msg)?),
        };

        Ok(this)
    }
}
