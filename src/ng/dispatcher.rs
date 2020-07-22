use super::{
    messages::{AllCommands, FromIrcMessage, ParseError},
    EventMap, EventStream, IrcMessage,
};

use std::sync::Arc;

#[derive(Debug)]
pub enum DispatchError {
    ParseError(ParseError),
    Foo,
}

impl From<()> for DispatchError {
    fn from(_: ()) -> Self {
        Self::Foo
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

    pub fn dispatch(&mut self, message: IrcMessage<'static>) -> Result<(), DispatchError> {
        match &*message.command {
            // "001" => self.dispatch_inner::<IrcReady>(message)?,
            // "376" => self.dispatch_inner::<Ready>(message)?,
            // "CAP" => self.dispatch_inner::<Cap>(message)?,
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
                self.dispatch_inner::<IrcMessage>(message.clone())
                    .expect("identity conversion should be upheld");
                self.dispatch_inner::<AllCommands>(message)
                    .expect("identity conversion should be upheld");
            }
        };

        Ok(())
    }

    pub fn reset(&mut self) {
        std::mem::take(&mut self.map);
    }

    fn dispatch_inner<T>(&mut self, message: IrcMessage<'static>) -> Result<(), T::Error>
    where
        T: FromIrcMessage<'static> + 'static,
        DispatchError: From<T::Error>,
    {
        let msg = T::from_irc(&message).map(Arc::new)?;
        self.map.send(msg);
        Ok(())
    }
}
