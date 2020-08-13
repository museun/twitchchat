use crate::{EventIter, EventStream, Sender};

use std::{
    any::{Any, TypeId},
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
};

type SenderList = Vec<(Id, Box<dyn Any + Send + Sync>)>;

/// The id of the mapped Sender
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Id(usize);

/// An event map which allows sending messages to a set of channels
#[derive(Default)]
pub struct EventMap {
    inner: HashMap<TypeId, SenderList>,
    id: usize,
}

impl EventMap {
    /// Create a new EventMap
    pub fn new() -> Self {
        Self::default()
    }

    /// Register this type with the EventMap, returning a clonable Receiver end
    pub fn register_stream<T>(&mut self) -> EventStream<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let rx = self.register::<T>();
        EventStream { inner: rx }
    }

    /// Register this type with the EventMap, returning a clonable Receiver end
    pub fn register_iter<T>(&mut self) -> EventIter<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let rx = self.register::<T>();
        EventIter { inner: rx }
    }

    /// Send this message to anything listening for it
    ///
    /// This will automatically clean up any stale senders after it fails to send
    pub fn send<T>(&mut self, msg: T) -> bool
    where
        T: Clone + Send + Sync + 'static,
    {
        // this ensures the index/unwrap below will always succeed
        if self.is_empty::<T>() {
            return false;
        }

        let handlers = self.inner[&TypeId::of::<T>()].iter().map(|(id, sender)| {
            // the type will always be valid, we don't expose the map
            let sender = sender.downcast_ref::<Sender<T>>().unwrap();
            (*id, sender)
        });

        let mut bad = BTreeSet::new();
        for (id, handler) in handlers {
            if let Err(..) = handler.send(msg.clone()) {
                // remove this id from the map
                bad.insert(id);
            }
        }
        self.remove::<T>(bad);

        true
    }

    /// Get the number of potential listeners for this message
    pub fn active<T: 'static>(&self) -> usize {
        self.inner
            .get(&TypeId::of::<T>())
            .map(Vec::len)
            .unwrap_or_default()
    }

    /// Determine whether there are any listeners for this message
    pub fn is_empty<T: 'static>(&self) -> bool {
        self.active::<T>() == 0
    }

    /// Get an iterator of all senders for this type
    ///
    /// The iterator will be over the `(Id, Sender<T>)`
    ///
    /// This returns None if no active senders are available
    pub fn get_senders<T>(&self) -> Option<Senders<'_, T>>
    where
        T: Send + Sync + 'static,
    {
        self.inner.get(&TypeId::of::<T>()).map(|list| Senders {
            inner: list.iter(),
            marker: PhantomData,
        })
    }

    /// Reset this EventMap, causing all pending Receivers to end.
    ///
    /// After doing this, you'll need to re-register all subscriptions
    pub fn reset(&mut self) {
        std::mem::take(&mut self.inner);
        self.id = 0;
    }

    fn remove<T: 'static>(&mut self, mut values: BTreeSet<Id>) {
        if values.is_empty() {
            // quick path because remove is called every dispatch
            return;
        }

        if let Some(inner) = self.inner.get_mut(&TypeId::of::<T>()) {
            // inverted so we remove them
            inner.retain(|(id, _)| !values.remove(id))
        }
    }

    fn register<T>(&mut self) -> async_channel::Receiver<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = crate::channel::unbounded();
        self.inner
            .entry(TypeId::of::<T>())
            .or_default()
            .push((Id(self.id), Box::new(tx)));
        self.id += 1;
        rx
    }
}

/// An iterator over Senders for this message
///
/// This produces the `Id` and the `Sender` for that type
pub struct Senders<'a, T: Send + Sync + 'static> {
    inner: std::slice::Iter<'a, (Id, Box<dyn Any + Send + Sync>)>,
    marker: PhantomData<T>,
}

impl<'a, T: Send + Sync + 'static> Iterator for Senders<'a, T> {
    type Item = (Id, Sender<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let (id, sender) = self.inner.next()?;
        // because we control the map, users cannot insert a non 'Sender<T>', so this unwrap is safe
        let sender = sender.downcast_ref::<Sender<T>>().cloned().unwrap();
        Some((*id, sender))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_map_senders_iter() {
        let mut map = EventMap::new();
        let _strings = map.register_iter::<&'static str>();
        let mut a = map.register_iter::<i32>();
        let mut b = map.register_iter::<i32>();

        assert!(map.get_senders::<usize>().is_none());

        for (_, sender) in map.get_senders::<i32>().unwrap() {
            sender.send(42).unwrap();
        }

        assert_eq!(a.next().unwrap(), 42);
        assert_eq!(b.next().unwrap(), 42);

        assert!(a.next().is_none());
        assert!(b.next().is_none());
    }

    #[test]
    fn event_map_async() {
        futures_lite::future::block_on(async move {
            use futures_lite::StreamExt as _;

            #[derive(Clone, Debug, PartialEq)]
            struct Message {
                data: String,
            }

            let mut map = EventMap::new();

            // subscriptions are initially empty
            assert_eq!(map.is_empty::<i32>(), true);
            assert_eq!(map.is_empty::<String>(), true);
            assert_eq!(map.is_empty::<Message>(), true);

            // subscribe two i32 twice
            assert_eq!(map.active::<i32>(), 0);
            let mut i1 = map.register_stream::<i32>();
            let mut i2 = map.register_stream::<i32>();
            // we should have 2 subscriptions active
            assert_eq!(map.active::<i32>(), 2);

            // subscribe to our custom type twice
            assert_eq!(map.active::<Message>(), 0);
            let mut m1 = map.register_stream::<Message>();
            let mut m2 = map.register_stream::<Message>();
            assert_eq!(map.active::<Message>(), 2);

            // send an i32
            assert_eq!(map.send(42_i32), true);

            // send our message
            let msg = Message {
                data: String::from("hello world"),
            };
            assert_eq!(map.send(msg.clone()), true);

            // will block until we get our number
            assert_eq!(i1.next().await.unwrap(), 42);
            // and the other one will also get it
            assert_eq!(i2.next().await.unwrap(), 42);

            // will block until we get our message
            assert_eq!(m1.next().await.unwrap(), msg);
            // and the other one will also get it
            assert_eq!(m2.next().await.unwrap(), msg);

            // no one is listening for () so this'll return false
            assert_eq!(map.send(()), false);

            // drop a subscription
            drop(i1);
            // and send a new value
            assert_eq!(map.send(43_i32), true);
            // only i2 will get it
            assert_eq!(i2.next().await.unwrap(), 43);
            // and it cleaned up the dropped subscription
            assert_eq!(map.active::<i32>(), 1);
        });
    }

    #[test]
    fn event_map() {
        #[derive(Clone, Debug, PartialEq)]
        struct Message {
            data: String,
        }

        let mut map = EventMap::new();

        // subscriptions are initially empty
        assert_eq!(map.is_empty::<i32>(), true);
        assert_eq!(map.is_empty::<String>(), true);
        assert_eq!(map.is_empty::<Message>(), true);

        // subscribe two i32 twice
        assert_eq!(map.active::<i32>(), 0);
        let mut i1 = map.register_iter::<i32>();
        let mut i2 = map.register_iter::<i32>();
        // we should have 2 subscriptions active
        assert_eq!(map.active::<i32>(), 2);

        // subscribe to our custom type twice
        assert_eq!(map.active::<Message>(), 0);
        let mut m1 = map.register_iter::<Message>();
        let mut m2 = map.register_iter::<Message>();
        assert_eq!(map.active::<Message>(), 2);

        // send an i32
        assert_eq!(map.send(42_i32), true);

        // send our message
        let msg = Message {
            data: String::from("hello world"),
        };
        assert_eq!(map.send(msg.clone()), true);

        // will block until we get our number
        assert_eq!(i1.next().unwrap(), 42);
        // and the other one will also get it
        assert_eq!(i2.next().unwrap(), 42);

        // will block until we get our message
        assert_eq!(m1.next().unwrap(), msg);
        // and the other one will also get it
        assert_eq!(m2.next().unwrap(), msg);

        // no one is listening for () so this'll return false
        assert_eq!(map.send(()), false);

        // drop a subscription
        drop(i1);
        // and send a new value
        assert_eq!(map.send(43_i32), true);
        // only i2 will get it
        assert_eq!(i2.next().unwrap(), 43);
        // and it cleaned up the dropped subscription
        assert_eq!(map.active::<i32>(), 1);
    }
}
