use super::{
    messages::{AllCommands, Cap, FromIrcMessage, InvalidMessage},
    AsOwned, EventMap, EventStream, IrcMessage,
};

use std::{convert::Infallible, sync::Arc};

#[derive(Debug)]
#[non_exhaustive]
pub enum DispatchError {
    InvalidMessage(InvalidMessage),
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

#[derive(Default)]
pub struct Dispatcher {
    map: EventMap,
}

impl Dispatcher {
    pub fn subscribe<T: 'static>(&mut self) -> EventStream<T> {
        let rx = self.map.register();
        EventStream { inner: rx }
    }

    pub fn dispatch<'a>(&mut self, message: &'a IrcMessage<'a>) -> Result<(), DispatchError> {
        match &*message.command {
            // "001" => self.dispatch_inner::<IrcReady>(message)?,
            // "376" => self.dispatch_inner::<Ready>(message)?,
            "CAP" => self.dispatch_inner::<Cap>(message)?,
            // "CLEARCHAT" => self.dispatch_inner::<ClearChat>(message)?,
            // "CLEARMSG" => self.dispatch_inner::<ClearMsg>(message)?,
            // "GLOBALUSERSTATE" => self.dispatch_inner::<GlobalUserState>(message)?,
            // "HOSTARGET" => self.dispatch_inner::<HostTarget>(message)?,
            // "JOIN" => self.dispatch_inner::<Join>(message)?,
            // "NOTICE" => self.dispatch_inner::<Notice>(message)?,
            // "PART" => self.dispatch_inner::<Part>(message)?,
            // "PING" => self.dispatch_inner::<Ping>(message)?,
            // "PONG" => self.dispatch_inner::<Pong>(message)?,
            // "PRIVMSG" => self.dispatch_inner::<Privmsg>(message)?,
            // "RECONNECT" => self.dispatch_inner::<Reconnect>(message)?,
            // "ROOMSTATE" => self.dispatch_inner::<RoomState>(message)?,
            // "USERNOTICE" => self.dispatch_inner::<UserNotice>(message)?,
            // "USERSTATE" => self.dispatch_inner::<UserState>(message)?,
            // "WHISPER" => self.dispatch_inner::<Whisper>(message)?,
            // TODO allow for user defined mappings
            _ => {
                // self.dispatch_inner::<IrcMessage>(message)
                //     .expect("identity conversion should be upheld");
                // self.dispatch_inner::<AllCommands>(message)
                //     .expect("identity conversion should be upheld");
            }
        };

        Ok(())
    }

    pub fn reset(&mut self) {
        std::mem::take(&mut self.map);
    }

    fn dispatch_inner<'a, T>(&mut self, message: &'a IrcMessage<'a>) -> Result<(), DispatchError>
    where
        T: FromIrcMessage<'a>,
        T: AsOwned + 'a,
        DispatchError: From<T::Error>,
    {
        let msg = T::from_irc(message)
            .map(|s| AsOwned::as_owned(&s))
            .map(Arc::new)?;

        self.map.send(msg);
        Ok(())
    }
}
