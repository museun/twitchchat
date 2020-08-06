use crate::{
    messages::*, EventMap, EventStream, FromIrcMessage, IntoOwned, InvalidMessage, IrcMessage,
};
use std::convert::Infallible;

/// An error produced by the Dispatcher
#[derive(Debug)]
#[non_exhaustive]
pub enum DispatchError {
    /// The message type was wrong -- this will only happen on user-defined events.
    InvalidMessage(InvalidMessage),
    /// A custom error message -- this will only happen on user-defined events.
    Custom(Box<dyn std::error::Error>),
}

impl DispatchError {
    /// Create a new custom error message type
    pub fn custom(err: impl std::error::Error + 'static) -> Self {
        Self::Custom(Box::new(err))
    }
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMessage(err) => write!(f, "invalid message: {}", err),
            Self::Custom(err) => write!(f, "unknown error: {}", err),
        }
    }
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidMessage(err) => Some(err),
            Self::Custom(err) => Some(&**err),
        }
    }
}

impl From<InvalidMessage> for DispatchError {
    fn from(msg: InvalidMessage) -> Self {
        Self::InvalidMessage(msg)
    }
}

impl From<Infallible> for DispatchError {
    fn from(_: Infallible) -> Self {
        unreachable!("you cannot produce this error")
    }
}

/// A message dispatcher
#[derive(Default)]
pub struct Dispatcher {
    map: EventMap,
}

impl Dispatcher {
    // Create a new message dispatcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to the provided message type, giving you an stream (and iterator) over any future messages.
    pub fn subscribe<T: Clone + 'static>(&mut self) -> EventStream<T> {
        self.map.register()
    }

    /// Dispatch this `IrcMessage`
    pub fn dispatch<'a>(&mut self, message: IrcMessage<'a>) -> Result<(), DispatchError> {
        use IrcMessage as M;

        let msg = message.into_owned();
        macro_rules! dispatch {
            ($ty:ty) => {
                self.dispatch_static::<$ty>(msg)?
            };
        }

        match msg.get_command() {
            M::IRC_READY => dispatch!(IrcReady),
            M::READY => dispatch!(Ready),
            M::CAP => dispatch!(Cap),
            M::CLEAR_CHAT => dispatch!(ClearChat),
            M::CLEAR_MSG => dispatch!(ClearMsg),
            M::GLOBAL_USER_STATE => dispatch!(GlobalUserState),
            M::HOST_TARGET => dispatch!(HostTarget),
            M::JOIN => dispatch!(Join),
            M::NOTICE => dispatch!(Notice),
            M::PART => dispatch!(Part),
            M::PING => dispatch!(Ping),
            M::PONG => dispatch!(Pong),
            M::PRIVMSG => dispatch!(Privmsg),
            M::RECONNECT => dispatch!(Reconnect),
            M::ROOM_STATE => dispatch!(RoomState),
            M::USER_NOTICE => dispatch!(UserNotice),
            M::USER_STATE => dispatch!(UserState),
            M::WHISPER => dispatch!(Whisper),
            _ => {
                // TODO user-defined messages

                self.dispatch_static::<IrcMessage>(msg.clone())
                    .expect("identity conversion should be upheld");

                self.dispatch_static::<AllCommands>(msg)
                    .expect("identity conversion should be upheld");
            }
        };

        Ok(())
    }

    /// Reset the dispatcher, this will cause all `EventStreams` previously produce via subscription to eventually return None.
    ///
    /// You'll have to re-subscribe to events after this.
    ///
    /// This is a way to stop any polling event handlers
    pub fn reset(&mut self) {
        self.map.reset()
    }

    fn dispatch_static<T>(&mut self, message: IrcMessage<'static>) -> Result<(), DispatchError>
    where
        T: FromIrcMessage<'static>,
        T: Clone + 'static,
        DispatchError: From<T::Error>,
    {
        self.map.send(T::from_irc(message)?);
        Ok(())
    }
}
