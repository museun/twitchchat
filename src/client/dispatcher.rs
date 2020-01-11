use super::{Event, EventStream};
use crate::decode::Message;
use crate::events;
use crate::Parse;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;

/**
An event dispatcher

This allows multiple sources to subscribe to specific [Events] which'll produce a corresponding [Message].

The subscription will return a [EventStream] which can be used as a [Stream].

[Events]: ../events/events/index.html
[Message]: ../events/messages/index.html
[EventStream]: ./struct.EventStream.html
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
*/
pub struct Dispatcher {
    event_map: HashMap<TypeId, Vec<(bool, Box<dyn Any + Send>)>>,
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher").finish()
    }
}

impl Dispatcher {
    /**
    Subscribe to an [Event] which'll return a [Stream] of a corresponding [Message].

    # Example
    ```rust,ignore
    # use twitchchat::events::{self, Dispatcher};
    // get some streams for events you're interested in
    // when you drop the streams it'll unsubscribe them
    let mut join_stream = dispatcher.subscribe::<events::Join>();
    let mut privmsg_stream = dispatcher.subscribe::<events::Privmsg>();
    // you can subscribe multiple times to the same event
    let mut another_one = dispatcher.subscribe::<events::Privmsg>();

    let print_joins = async move {
        // loop over the stream printing out the messages
        // the message type will be JoinMessage here.
        while let Some(msg) = join_stream.next().await {
            println!("{:?}", msg);
        }
    };
    ```
    # Mapping
    Use an event from [Events][Event] and subscribe will produce an [`EventStream<Arc<T::Mapped>>`][EventStream] which corresponds to the message in [Messages][Message].

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
    [Mode][Mode_event]                       | [Mode][Mode_message]                       | When someone gains/loses moderator status
    [Notice][Notice_event]                   | [Notice][Notice_message]                   | General notices from the server
    [Part][Part_event]                       | [Part][Part_message]                       | When a user leaves a channel
    [Ping][Ping_event]                       | [Ping][Ping_message]                       | Server requesting a response (for heartbeat/connection tracking)
    [Pong][Pong_event]                       | [Pong][Pong_message]                       | Server responding to a user-provided PING
    [Privmsg][Privmsg_event]                 | [Privmsg][Privmsg_message]                 | A normal message sent by a user
    [Raw][Raw_event]                         | [Raw][Raw_message]                         | The raw message before being specialized
    [Ready][Ready_event]                     | [Ready][Ready_message]                     | When the Twitch connection is ready
    [Reconnect][Reconnect_event]             | [Reconnect][Reconnect_message]             | Server asking you to reconnect
    [UserState][UserState_event]             | [UserState][UserState_message]             | Identifies a user's chat settings or properties (e.g., chat color).

    # Disconnection
    The stream will also yield ***None*** when the `Dispatcher` is dropped.

    Or if the subscriptions were cleared.

    [Event]: ../events/events/index.html
    [Message]: ../events/messages/index.html
    [EventStream]: ../events/struct.EventStream.html
    [Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html

    [Cap_event]: ../events/struct.Cap.html
    [ClearChat_event]: ../events/struct.ClearChat.html
    [ClearMsg_event]: ../events/struct.ClearMsg.html
    [GlobalUserState_event]: ../events/struct.GlobalUserState.html
    [HostTarget_event]: ../events/struct.HostTarget.html
    [IrcReady_event]: ../events/struct.IrcReady.html
    [Join_event]: ../events/struct.Join.html
    [Mode_event]: ../events/struct.Mode.html
    [Notice_event]: ../events/struct.Notice.html
    [Part_event]: ../events/struct.Part.html
    [Ping_event]: ../events/struct.Ping.html
    [Pong_event]: ../events/struct.Pong.html
    [Privmsg_event]: ../events/struct.Privmsg.html
    [Raw_event]: ../events/struct.Raw.html
    [Ready_event]: ../events/struct.Ready.html
    [Reconnect_event]: ../events/struct.Reconnect.html
    [UserState_event]: ../events/struct.UserState.html

    [Cap_message]: ../messages/struct.Cap.html
    [ClearChat_message]: ../messages/struct.ClearChat.html
    [ClearMsg_message]: ../messages/struct.ClearMsg.html
    [GlobalUserState_message]: ../messages/struct.GlobalUserState.html
    [HostTarget_message]: ../messages/struct.HostTarget.html
    [IrcReady_message]: ../messages/struct.IrcReady.html
    [Join_message]: ../messages/struct.Join.html
    [Mode_message]: ../messages/struct.Mode.html
    [Notice_message]: ../messages/struct.Notice.html
    [Part_message]: ../messages/struct.Part.html
    [Ping_message]: ../messages/struct.Ping.html
    [Pong_message]: ../messages/struct.Pong.html
    [Privmsg_message]: ../messages/struct.Privmsg.html
    [Raw_message]: ../messages/struct.Raw.html
    [Ready_message]: ../messages/struct.Ready.html
    [Reconnect_message]: ../messages/struct.Reconnect.html
    [UserState_message]: ../messages/struct.UserState.html

    */
    pub fn subscribe<'a, T>(&mut self) -> EventStream<Arc<T::Mapped>>
    where
        T: Event<'a> + 'static,
    {
        self.subscribe_internal::<T>(false)
    }

    /// Allows marking a subscription as internal
    ///
    /// Internal subscriptions can't be removed by the user
    pub(crate) fn subscribe_internal<'a, T>(&mut self, private: bool) -> EventStream<Arc<T::Mapped>>
    where
        T: Event<'a> + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_map
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .push((private, Box::new(Sender::new(tx))));
        let name = std::any::type_name::<T>().split("::").last().unwrap();
        if !private {
            log::debug!("adding subscription: {}", name);
        } else {
            log::trace!("adding internal subscription: {}", name);
        }
        EventStream(rx)
    }

    /// Get the subscriber count for a specific event
    pub fn count_subscribers<'a, T>(&self) -> usize
    where
        T: Event<'a> + 'static,
    {
        self.event_map
            .get(&TypeId::of::<T>())
            .map(|s| s.iter().filter(|&(private, _)| !private).count())
            .unwrap_or_default()
    }

    /// Get the subscriber count for all events
    pub fn count_subscribers_all(&self) -> usize {
        self.event_map
            .values()
            .map(|s| s.iter().filter(|&(private, _)| !private).count())
            .sum()
    }

    /// Clear subscriptions for a specific event, returning how many subscribers were removed
    pub fn clear_subscriptions<'a, T>(&mut self) -> usize
    where
        T: Event<'a> + 'static,
    {
        let n = self
            .event_map
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
    pub fn clear_subscriptions_all(&mut self) -> usize {
        let n = self
            .event_map
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

    /// Add this event into the dispatcher
    pub(crate) fn add_event<'a, T>(mut self) -> Self
    where
        T: Event<'a> + 'static,
    {
        self.event_map.entry(TypeId::of::<T>()).or_default();
        self
    }

    /// Tries to send this message to any subscribers
    pub(crate) fn try_send<'a, T>(&mut self, msg: &'a Message<&'a str>)
    where
        T: Event<'a> + 'static,
    {
        if let Some(senders) = self
            .event_map
            .get_mut(&TypeId::of::<T>())
            .filter(|s| !s.is_empty())
        {
            let msg = T::Mapped::parse(msg).map(Arc::new).expect("valid message");

            senders.retain(|(_, sender)| {
                sender
                    .downcast_ref::<Sender<T::Mapped>>()
                    .unwrap()
                    .try_send(Arc::clone(&msg))
            });
        }
    }
}

impl Dispatcher {
    make_mapping! {
        "001"             => IrcReady
        "PING"            => Ping
        "PONG"            => Pong
        "376"             => Ready
        "JOIN"            => Join
        "PART"            => Part
        "PRIVMSG"         => Privmsg
        "CAP"             => Cap
        "HOSTARGET"       => HostTarget
        "GLOBALUSERSTATE" => GlobalUserState
        "NOTICE"          => Notice
        "CLEARCHAT"       => ClearChat
        "CLEARMSG"        => ClearMsg
        "RECONNECT"       => Reconnect
        "USERSTATE"       => UserState
        "MODE"            => Mode
    }
}

struct Sender<T> {
    sender: mpsc::UnboundedSender<Arc<T>>,
}

impl<T> Sender<T> {
    fn new(sender: mpsc::UnboundedSender<Arc<T>>) -> Self {
        Self { sender }
    }

    fn try_send(&self, msg: Arc<T>) -> bool {
        self.sender.send(msg).is_ok()
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::*;

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

        let mut dispatcher = Dispatcher::new();
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

        let msg = crate::decode("foobar\r\n").map(|(_, msg)| msg).unwrap();

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
