cfg_async! { use futures_lite::{AsyncRead, AsyncWrite}; }
use std::io::{Read, Write};

use super::Error;

cfg_async! { use crate::AsyncDecoder; }
use crate::{
    util::is_blocking_error, //
    DecodeError,
    Decoder,
    IntoOwned as _,
    IrcMessage,
};

/// An event to wait for
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Event {
    /// Raw event
    Raw,
    /// IrcReady event
    IrcReady,
    /// Ready event
    Ready,
    /// Capabilities event
    Cap,
    /// ClearChat event
    ClearChat,
    /// ClearMsg event
    ClearMsg,
    /// GlobalUserState event
    GlobalUserState,
    /// HostTarget event
    HostTarget,
    /// Join event
    Join,
    /// Notice event
    Notice,
    /// Part event
    Part,
    /// Ping event
    Ping,
    /// Pong event
    Pong,
    /// Privmsg event
    Privmsg,
    /// Reconnect event
    Reconnect,
    /// RoomState event
    RoomState,
    /// UserNotice event
    UserNotice,
    /// UserState event
    UserState,
    /// Whisper event
    Whisper,
}

impl Event {
    fn from_raw(s: &str) -> Self {
        use IrcMessage as M;
        match s {
            M::IRC_READY => Self::IrcReady,
            M::READY => Self::Ready,
            M::CAP => Self::Cap,
            M::CLEAR_CHAT => Self::ClearChat,
            M::CLEAR_MSG => Self::ClearMsg,
            M::GLOBAL_USER_STATE => Self::GlobalUserState,
            M::HOST_TARGET => Self::HostTarget,
            M::JOIN => Self::Join,
            M::NOTICE => Self::Notice,
            M::PART => Self::Part,
            M::PING => Self::Ping,
            M::PONG => Self::Pong,
            M::PRIVMSG => Self::Privmsg,
            M::RECONNECT => Self::Reconnect,
            M::ROOM_STATE => Self::RoomState,
            M::USER_NOTICE => Self::UserNotice,
            M::USER_STATE => Self::UserState,
            M::WHISPER => Self::Whisper,
            _ => Self::Raw,
        }
    }
}

/// Wait for this specific [`Event`], using the provided [`Decoder`]
///
/// This'll block until it finds a matching [`Event`]
///
/// On success it'll return the [`IrcMessage`] and any messages you missed
///
/// You can use [`std::iter::Extend`] to feed these messages back into the [`Decoder`], if you want to replay things.
///
/// ```no_run
/// # use twitchchat::{*, runner::*};
/// # let stream = std::io::Cursor::new(Vec::new());
/// # let mut decoder = twitchchat::Decoder::new(stream);
/// use twitchchat::FromIrcMessage as _;
/// // this will block until you get a 'ClearMsg'
/// let (msg, missed_messages) = wait_for_sync(Event::ClearMsg, &mut decoder).unwrap();
/// // and then you can convert it to that type
/// let msg = messages::ClearMsg::from_irc(msg).unwrap();
/// ```
pub fn wait_for_sync<I>(
    event: Event,
    dec: &mut Decoder<I>,
) -> Result<(IrcMessage<'static>, Vec<IrcMessage<'static>>), Error>
where
    I: Read + Write,
{
    let mut missed = vec![];
    loop {
        match wait_inner(dec.read_message(), event)? {
            Done::Yep(msg) => break Ok((msg.into_owned(), missed)),
            Done::Nope(msg) => missed.push(msg.into_owned()),
            Done::Yield => std::thread::yield_now(),
        }
    }
}

cfg_async! {
/// Wait for this specific [`Event`], using the provided [`AsyncDecoder`]
///
/// This'll block until it finds a matching [`Event`]
///
/// On success it'll return the [`IrcMessage`] and any messages you missed
///
/// You can use [`std::iter::Extend`] to feed these messages back into the [`AsyncDecoder`], if you want to replay things.
///
/// ```no_run
/// # use twitchchat::{*, runner::*};
/// # let stream = futures_lite::io::Cursor::new(Vec::new());
/// # let mut decoder = twitchchat::AsyncDecoder::new(stream);
/// use twitchchat::FromIrcMessage as _;
/// # let fut = async {
/// // this will block until you get a 'ClearMsg'
/// let (msg, missed_messages) = wait_for(Event::ClearMsg, &mut decoder).await.unwrap();
/// // and then you can convert it to that type
/// let msg = messages::ClearMsg::from_irc(msg).unwrap();
/// # };
/// ```
pub async fn wait_for<I>(
    event: Event,
    dec: &mut AsyncDecoder<I>,
) -> Result<(IrcMessage<'static>, Vec<IrcMessage<'static>>), Error>
where
    I: AsyncRead + AsyncWrite + Send + Sync + Unpin,
{
    let mut missed = vec![];
    loop {
        match wait_inner(dec.read_message().await, event)? {
            Done::Yep(msg) => break Ok((msg.into_owned(), missed)),
            Done::Nope(msg) => missed.push(msg.into_owned()),
            Done::Yield => futures_lite::future::yield_now().await,
        }
    }
}
}

enum Done<'a> {
    Yep(IrcMessage<'a>),
    Nope(IrcMessage<'a>),
    Yield,
}

fn wait_inner(
    result: Result<IrcMessage<'_>, DecodeError>,
    event: Event,
) -> Result<Done<'_>, Error> {
    let msg = match result {
        Err(DecodeError::Io(err)) if is_blocking_error(&err) => return Ok(Done::Yield),
        Err(err) => return Err(err.into()),
        Ok(msg) => msg,
    };

    if Event::from_raw(msg.get_command()) == event {
        Ok(Done::Yep(msg))
    } else {
        Ok(Done::Nope(msg))
    }
}
