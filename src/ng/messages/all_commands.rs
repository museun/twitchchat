use super::*;
use crate::ng::{FromIrcMessage, InvalidMessage};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum AllCommands<'a> {
    Raw(IrcMessage<'a>),
    IrcReady(IrcReady<'a>),
    Ready(Ready<'a>),
    Cap(Cap<'a>),
    ClearChat(ClearChat<'a>),
    ClearMsg(ClearMsg<'a>),
    GlobalUserState(GlobalUserState<'a>),
    HostTarget(HostTarget<'a>),
    Join(Join<'a>),
    Notice(Notice<'a>),
    Part(Part<'a>),
    Ping(Ping<'a>),
    Pong(Pong<'a>),
    Privmsg(Privmsg<'a>),
    Reconnect(Reconnect<'a>),
    RoomState(RoomState<'a>),
    UserNotice(UserNotice<'a>),
    UserState(UserState<'a>),
    Whisper(Whisper<'a>),
}

impl<'a> FromIrcMessage<'a> for AllCommands<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        macro_rules! map {
            ($ident:ident) => {
                Self::$ident($ident::from_irc(msg)?)
            };
        }

        use IrcMessage as M;
        let this = match msg.get_command() {
            M::IRCREADY => map!(IrcReady),
            M::READY => map!(Ready),
            M::CAP => map!(Cap),
            M::CLEARCHAT => map!(ClearChat),
            M::CLEARMSG => map!(ClearMsg),
            M::GLOBALUSERSTATE => map!(GlobalUserState),
            M::HOSTTARGET => map!(HostTarget),
            M::JOIN => map!(Join),
            M::NOTICE => map!(Notice),
            M::PART => map!(Part),
            M::PING => map!(Ping),
            M::PONG => map!(Pong),
            M::PRIVMSG => map!(Privmsg),
            M::RECONNECT => map!(Reconnect),
            M::ROOMSTATE => map!(RoomState),
            M::USERNOTICE => map!(UserNotice),
            M::USERSTATE => map!(UserState),
            M::WHISPER => map!(Whisper),
            _ => Self::Raw(IrcMessage::from_irc(msg).expect("infallible conversion")),
        };

        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn all_commands_serde() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        crate::ng::round_trip_json::<AllCommands>(input);
    }

    #[test]
    fn ensure_const_match() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        let msg = IrcMessage::parse(Str::Borrowed(input));
        let all = AllCommands::from_irc(msg).unwrap();
        assert!(matches!(all, AllCommands::Privmsg{..}));
    }
}
