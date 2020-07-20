use super::{Event, EventMapped};
use crate::decode::Message;
use crate::events;
use crate::{Error, EventStream, Parse};

use futures_lite::StreamExt;
use parking_lot::Mutex;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

type EventRegistration = Vec<(bool, Box<dyn Any + Send>)>;

type AnyMap<T> = Arc<Mutex<HashMap<TypeId, T>>>;

/**
An event dispatcher

This allows multiple sources to subscribe to specific [Events] which'll produce a corresponding [Message].

The subscription will return a [EventStream] which can be used as a [Stream].

[Events]: ./events/index.html
[Message]: ./messages/index.html
[EventStream]: ./struct.EventStream.html
[Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html
*/
#[derive(Clone)]
pub struct Dispatcher {
    // TODO this should be using a normal mutex, its locking time is generally
    // just for a single insertion into a hashmap so it won't block the executor
    pub(crate) event_map: AnyMap<EventRegistration>,
    cached: AnyMap<Box<dyn Any + Send>>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        use crate::events::*;

        let mut event_map = HashMap::default();

        macro_rules! add {
            ($($event:ty)*) => {
                $(
                event_map
                    .entry(std::any::TypeId::of::<$event>())
                    .or_default();
                )*
            };
        }

        add! {
            IrcReady
            Ready
            Cap
            ClearChat
            ClearMsg
            GlobalUserState
            HostTarget
            Join
            Notice
            Part
            Ping
            Pong
            Privmsg
            Reconnect
            RoomState
            UserState
            UserNotice
            Whisper
            // the meta-events
            All
            Raw
        }

        Self {
            event_map: Arc::new(Mutex::new(event_map)),
            cached: Default::default(),
        }
    }
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher").finish()
    }
}

impl Dispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self::default()
    }

    /** Subscribes to an event and blocks until the next item is available

    This is useful when you want to wait, for say, the IrcReady event before you join channels.

    ---
    ***NOTE*** Any subsequent calls to `wait_for` for this event will return a _cached_ value.

    # Example
    ```rust
    # use twitchchat::{Dispatcher, Runner, RateLimit, Connector, events};
    # use tokio::spawn;
    # use futures::prelude::*;
    # let data = b":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
    # let conn = Connector::new(move || async move { Ok(tokio_test::io::Builder::new().read(data).wait(std::time::Duration::from_millis(10000)).build()) });
    # let fut = async move {
    let dispatcher = Dispatcher::new();
    let (mut runner, control) = Runner::new(dispatcher.clone());
    // You should spawn the run() away so it can start to process events
    let handle = spawn(async move { runner.run_to_completion(conn).await });
    // block until we get an IrcReady
    let _ = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
    # assert!(true);
    // it'll cache the event
    let _ = dispatcher.wait_for::<events::IrcReady>()
        .now_or_never()
        .expect("cached value")
        .unwrap();
    # assert!(true);
    // stop the runner
    control.stop();
    // just to wait for the spawned task to end
    let _ = handle.await.unwrap().unwrap();
    # };
    # tokio::runtime::Runtime::new().unwrap().block_on(fut);
    ```
    */
    pub async fn wait_for<T>(&self) -> Result<Arc<T::Owned>, Error>
    where
        T: Event<'static> + 'static,
        T: EventMapped<'static, T>,
    {
        if let Some(item) = self
            .cached
            .lock()
            .get(&TypeId::of::<T>())
            .map(|s| Arc::clone(s.downcast_ref::<Arc<T::Owned>>().expect("valid type")))
        {
            log::debug!("got a cached event for: {}", std::any::type_name::<T>());
            return Ok(item);
        }

        let item = self
            .subscribe_internal::<T>(false)
            .next()
            .await
            .ok_or_else(|| Error::ClientDisconnected)?;

        self.cached
            .lock()
            .insert(TypeId::of::<T>(), Box::new(Arc::clone(&item)));

        log::debug!("got an event for: {}", std::any::type_name::<T>());

        Ok(item)
    }

    /**
    Subscribe to an [Event] which'll return a [Stream] of a corresponding [Message].

    # Example
    ```rust
    # use twitchchat::{Dispatcher, Runner, Connector, events, RateLimit};
    # use tokio::spawn;
    # use futures::prelude::*;
    # let data = b":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
    # let conn = Connector::new(move || async move { Ok(tokio_test::io::Builder::new().read(data).wait(std::time::Duration::from_millis(1000)).build()) });
    # let fut = async move {
    let dispatcher = Dispatcher::new();
    let (mut runner, control) = Runner::new(dispatcher.clone());
    // spawn the runner in the background, just to drive things for us
    // (you could select over it, or await at the end)
    spawn(async move { runner.run_to_completion(conn).await });
    # control.stop(); // this is just so things will stop now

    // get some streams for events you're interested in
    // when you drop the streams it'll unsubscribe them
    let mut join_stream = dispatcher.subscribe::<events::Join>();
    let privmsg_stream = dispatcher.subscribe::<events::Privmsg>();
    // you can subscribe multiple times to the same event
    let another_one = dispatcher.subscribe::<events::Privmsg>();
    // you can also get an enum of all possible events
    let mut all_events = dispatcher.subscribe::<events::All>();
    // or the raw IRC message
    let raw_event = dispatcher.subscribe::<events::Raw>();

    // using for each
    let print_raw = raw_event.for_each(|msg| async move {
        println!("{}", msg.raw.escape_debug());
    });
    // and spawn that future on another task
    spawn(print_raw);

    // loop and select
    loop {
        tokio::select!{
            Some(msg) = &mut join_stream.next() => {}
            Some(all) = &mut all_events.next() => {}
            else => break
        }
    }
    # };
    # tokio::runtime::Runtime::new().unwrap().block_on(fut);
    ```

    # Mapping
    Use an event from [Events][Event] and subscribe will produce an [`EventStream<Arc<T>>`][EventStream] which corresponds to the message in [Messages][Message].

    ## A table of mappings
    Event                                    | Message                                    | Description
    ---                                      | ---                                        | ---
    [Cap][Cap_event]                         | [Cap][Cap_message]                         | Capability response from the server
    [ClearChat][ClearChat_event]             | [ClearChat][ClearChat_message]             | Someone cleared the chat
    [ClearMsg][ClearMsg_event]               | [ClearMsg][ClearMsg_message]               | Someone removed a users message(s)
    [GlobalUserState][GlobalUserState_event] | [GlobalUserState][GlobalUserState_message] | Your user information from the server
    [HostTarget][HostTarget_event]           | [HostTarget][HostTarget_message]           | When a channel hosts/unhosts another channel
    [IrcReady][IrcReady_event]               | [IrcReady][IrcReady_message]               | When the IRC connection is ready
    [Join][Join_event]                       | [Join][Join_message]                       | When a user joins a channel
    [Notice][Notice_event]                   | [Notice][Notice_message]                   | General notices from the server
    [Part][Part_event]                       | [Part][Part_message]                       | When a user leaves a channel
    [Ping][Ping_event]                       | [Ping][Ping_message]                       | Server requesting a response (for heartbeat/connection tracking)
    [Pong][Pong_event]                       | [Pong][Pong_message]                       | Server responding to a user-provided PING
    [Privmsg][Privmsg_event]                 | [Privmsg][Privmsg_message]                 | A normal message sent by a user
    [Raw][Raw_event]                         | [Raw][Raw_message]                         | The raw message before being specialized
    [Ready][Ready_event]                     | [Ready][Ready_message]                     | When the Twitch connection is ready
    [Reconnect][Reconnect_event]             | [Reconnect][Reconnect_message]             | Server asking you to reconnect
    [RoomState][RoomState_event]             | [RoomState][RoomState_message]             | Server giving you information about the room
    [UserNotice][UserNotice_event]           | [UserNotice][UserNotice_message]           | Metadata attached to an user event (e.g. a subscription)
    [UserState][UserState_event]             | [UserState][UserState_message]             | Identifies a user's chat settings or properties (e.g., chat color).
    ---                                      | ---                                        | ---
    [All][All_event]                         | [AllCommands][AllCommands_message]         | This bundles all above messages into a single enum.

    # Disconnection
    The stream will also yield ***None*** when the `Dispatcher` is dropped.

    Or if the subscriptions were cleared.

    ## Tip
    If you hold onto clones of the dispatcher, you can remove the event, or all events to force the respective Stream(s) to end


    [Event]: ./events/index.html
    [Message]: ./messages/index.html
    [EventStream]: ./struct.EventStream.html
    [Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html

    [Cap_event]: ./events/struct.Cap.html
    [ClearChat_event]: ./events/struct.ClearChat.html
    [ClearMsg_event]: ./events/struct.ClearMsg.html
    [GlobalUserState_event]: ./events/struct.GlobalUserState.html
    [HostTarget_event]: ./events/struct.HostTarget.html
    [IrcReady_event]: ./events/struct.IrcReady.html
    [Join_event]: ./events/struct.Join.html
    [Mode_event]: ./events/struct.Mode.html
    [Part_event]: ./events/struct.Part.html
    [Ping_event]: ./events/struct.Ping.html
    [Pong_event]: ./events/struct.Pong.html
    [Privmsg_event]: ./events/struct.Privmsg.html
    [Raw_event]: ./events/struct.Raw.html
    [Ready_event]: ./events/struct.Ready.html
    [Reconnect_event]: ./events/struct.Reconnect.html
    [RoomState_event]: ./events/struct.RoomState.html
    [UserNotice_event]: ./events/struct.UserNotice.html
    [UserState_event]: ./events/struct.UserState.html
    [All_event]: ./events/struct.All.html

    [Cap_message]: ./messages/struct.Cap.html
    [ClearChat_message]: ./messages/struct.ClearChat.html
    [ClearMsg_message]: ./messages/struct.ClearMsg.html
    [GlobalUserState_message]: ./messages/struct.GlobalUserState.html
    [HostTarget_message]: ./messages/struct.HostTarget.html
    [IrcReady_message]: ./messages/struct.IrcReady.html
    [Join_message]: ./messages/struct.Join.html
    [Mode_message]: ./messages/struct.Mode.html
    [Part_message]: ./messages/struct.Part.html
    [Ping_message]: ./messages/struct.Ping.html
    [Pong_message]: ./messages/struct.Pong.html
    [Privmsg_message]: ./messages/struct.Privmsg.html
    [Raw_message]: ./messages/type.Raw.html
    [Ready_message]: ./messages/struct.Ready.html
    [Reconnect_message]: ./messages/struct.Reconnect.html
    [RoomState_message]: ./messages/struct.RoomState.html
    [UserNotice_message]: ./messages/struct.UserNotice.html
    [UserState_message]: ./messages/struct.UserState.html
    [AllCommands_message]: ./messages/enum.AllCommands.html
    */
    pub fn subscribe<'a, T>(&self) -> EventStream<Arc<T::Owned>>
    where
        T: Event<'a> + 'static,
        T: EventMapped<'a, T>,
    {
        self.subscribe_internal::<T>(false)
    }

    /// Allows marking a subscription as internal
    ///
    /// Internal subscriptions can't be removed by the user
    pub(crate) fn subscribe_internal<'a, T>(&self, private: bool) -> EventStream<Arc<T::Owned>>
    where
        T: Event<'a> + 'static,
        T: EventMapped<'a, T>,
    {
        let (tx, rx) = async_channel::unbounded::<Arc<T::Owned>>();
        self.event_map
            .lock()
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .push((private, Box::new(Sender::new(tx))));

        let name = std::any::type_name::<T>().split("::").last().unwrap();
        if !private {
            log::debug!("adding subscription: {}", name);
        } else {
            log::trace!("adding internal subscription: {}", name);
        }

        EventStream { inner: rx }
    }

    /// Get the subscriber count for a specific event
    pub fn count_subscribers<'a, T>(&self) -> usize
    where
        T: Event<'a> + 'static,
    {
        self.event_map
            .lock()
            .get(&TypeId::of::<T>())
            .map(|s| s.iter().filter(|&(private, _)| !private).count())
            .unwrap_or_default()
    }

    /// Get the subscriber count for all events
    pub fn count_subscribers_all(&self) -> usize {
        self.event_map
            .lock()
            .values()
            .map(|s| s.iter().filter(|&(private, _)| !private).count())
            .sum()
    }

    /// Reset the internal subscriptions. This prevents a resource leak in the retry-connect loop.
    pub(crate) fn reset_internal_subscriptions(&self) {
        // clear the cache
        self.cached.lock().clear();

        for value in self.event_map.lock().values_mut() {
            value.retain(|(i, _)| !i);
        }
    }

    /// Clear subscriptions for a specific event, returning how many subscribers were removed
    pub fn clear_subscriptions<'a, T>(&self) -> usize
    where
        T: Event<'a> + 'static,
    {
        let n = self
            .event_map
            .lock()
            .get_mut(&TypeId::of::<T>())
            .map(|list| {
                let old = list.len();
                list.retain(|&(private, _)| private);
                old - list.len()
            })
            .unwrap();

        let ty = std::any::type_name::<T>().split("::").last().unwrap();
        log::debug!("cleared {} subscriptions for {}", n, ty);
        n
    }

    /// Clear all subscriptions, returning how many subscribers were removed
    pub fn clear_subscriptions_all(&self) -> usize {
        let n = self
            .event_map
            .lock()
            .values_mut()
            .map(|list| {
                let old = list.len();
                list.retain(|&(private, _)| private);
                old - list.len()
            })
            .sum();
        log::debug!("cleared all subscriptions. total: {}", n);
        n
    }

    /// Tries to send this message to any subscribers
    pub(crate) fn try_send<'a, T>(&self, msg: &'a Message<'a>)
    where
        T: Event<'a> + 'static,
        T: EventMapped<'a, T>,
    {
        if let Some(senders) = self
            .event_map
            .lock()
            .get_mut(&TypeId::of::<T>())
            .filter(|s| !s.is_empty())
        {
            let msg = T::Parsed::parse(msg);
            let msg: Arc<T::Owned> = match msg {
                Ok(msg) => Arc::new(T::into_owned(msg)),
                Err(err) => {
                    log::error!("cannot parse message: {}. this is a bug.", err);
                    return;
                }
            };

            senders.retain(|(_, sender)| {
                sender
                    .downcast_ref::<Sender<T::Owned>>()
                    // TODO, if we remove this unwrap then we can expose the Event traits
                    .unwrap()
                    .try_send(Arc::clone(&msg))
            });
        }
    }
}

impl Dispatcher {
    pub(crate) fn dispatch<'a>(&self, msg: &'a Message<'a>) {
        macro_rules! try_send {
            ($ident:ident) => {{
                self.try_send::<events::$ident>(&msg)
            }};
        }

        match msg.command.as_ref() {
            "001" => try_send!(IrcReady),
            "376" => try_send!(Ready),
            "CAP" => try_send!(Cap),
            "CLEARCHAT" => try_send!(ClearChat),
            "CLEARMSG" => try_send!(ClearMsg),
            "GLOBALUSERSTATE" => try_send!(GlobalUserState),
            "HOSTARGET" => try_send!(HostTarget),
            "JOIN" => try_send!(Join),
            "NOTICE" => try_send!(Notice),
            "PART" => try_send!(Part),
            "PING" => try_send!(Ping),
            "PONG" => try_send!(Pong),
            "PRIVMSG" => try_send!(Privmsg),
            "RECONNECT" => try_send!(Reconnect),
            "ROOMSTATE" => try_send!(RoomState),
            "USERNOTICE" => try_send!(UserNotice),
            "USERSTATE" => try_send!(UserState),
            "WHISPER" => try_send!(Whisper),
            _ => {}
        }

        try_send!(All);
        try_send!(Raw);
    }
}

struct Sender<T> {
    sender: async_channel::Sender<Arc<T>>,
}

impl<T> Sender<T> {
    const fn new(sender: async_channel::Sender<Arc<T>>) -> Self {
        Self { sender }
    }

    fn try_send(&self, msg: Arc<T>) -> bool {
        self.sender.try_send(msg).is_ok()
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wait_for() {
        use crate::{Runner, Status};
        use futures::future::FutureExt as _;

        let data = b":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";

        let connector = crate::Connector::new(move || async move {
            Ok(tokio_test::io::Builder::new()
                .read(data)
                .wait(std::time::Duration::from_millis(100))
                .build())
        });

        let dispatcher = Dispatcher::new();

        let (mut runner, control) = Runner::new(dispatcher.clone());
        let handle = tokio::spawn(async move { runner.run_to_completion(connector).await });

        let _ = dispatcher.wait_for::<events::IrcReady>().await.unwrap();

        let _ = dispatcher
            .wait_for::<events::IrcReady>()
            .now_or_never()
            .unwrap()
            .unwrap();

        control.stop();

        assert_eq!(handle.await.unwrap().unwrap(), Status::Canceled);
    }

    #[tokio::test]
    async fn wait_for_never() {
        use crate::{Runner, Status};
        use futures::future::FutureExt as _;

        let data = b":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";

        let connector = crate::Connector::new(move || async move {
            Ok(tokio_test::io::Builder::new()
                .read(data)
                .wait(std::time::Duration::from_millis(100))
                .build())
        });

        let dispatcher = Dispatcher::new();

        let (mut runner, control) = Runner::new(dispatcher.clone());
        let handle = tokio::spawn(async move { runner.run_to_completion(connector).await });

        assert!(dispatcher
            .wait_for::<events::Join>()
            .now_or_never()
            .is_none());

        control.stop();

        assert_eq!(handle.await.unwrap().unwrap(), Status::Canceled);
    }

    #[test]
    fn zombie() {
        #[derive(Default)]
        struct Counter {
            keep: usize,
            temp: usize,
        }

        use std::sync::{Arc, Mutex};
        let counter: Arc<Mutex<Counter>> = Default::default();

        let (mut tick_tx, mut tick_rx) = tokio::sync::mpsc::channel::<()>(1);

        let dispatcher = Dispatcher::new();
        let mut keep = dispatcher.subscribe::<events::Raw>();
        let keep = {
            let counter = Arc::clone(&counter);
            async move {
                while let Some(..) = keep.next().await {
                    counter.lock().unwrap().keep += 1;
                    if tick_tx.send(()).await.is_err() {
                        break;
                    }
                }
            }
        };

        let mut temporal = dispatcher.subscribe::<events::Raw>();
        let temporal = {
            let counter = Arc::clone(&counter);
            async move {
                temporal.next().await;
                counter.lock().unwrap().temp += 1
            }
        };

        let msg = crate::decode_one("foobar\r\n").map(|(_, msg)| msg).unwrap();

        let test = async move {
            let keep = tokio::task::spawn(keep);
            let temporal = tokio::task::spawn(temporal);

            // send the messages out
            dispatcher.dispatch(&msg);

            // we should still have subscribers
            assert_eq!(dispatcher.count_subscribers::<events::Raw>(), 2);

            // have it subscribe by awaiting the task
            temporal.await.unwrap();

            {
                let _ = tick_rx.recv().await;
                let counter = counter.lock().unwrap();
                assert_eq!(counter.temp, 1);
                assert_eq!(counter.keep, 1);
            }

            // and one should be removed here
            dispatcher.dispatch(&msg);
            assert_eq!(dispatcher.count_subscribers::<events::Raw>(), 1);

            {
                let _ = tick_rx.recv().await;
                let counter = counter.lock().unwrap();
                assert_eq!(counter.temp, 1);
                assert_eq!(counter.keep, 2);
            }

            // clean up
            dispatcher.clear_subscriptions_all();
            assert_eq!(dispatcher.count_subscribers::<events::Raw>(), 0);

            keep.await.unwrap();

            {
                let _ = tick_rx.recv().await;
                let counter = counter.lock().unwrap();
                assert_eq!(counter.temp, 1);
                assert_eq!(counter.keep, 2);
            }
        };

        tokio::runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()
            .unwrap()
            .block_on(test);
    }
}
*/
