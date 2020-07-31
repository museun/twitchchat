use super::{channel, Receiver, Sender};

use async_channel::TrySendError;
use std::{
    any::{Any, TypeId},
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Id(usize);

type SenderList = Vec<(Id, Box<dyn Any>)>;

#[derive(Default)]
pub struct EventMap {
    inner: HashMap<TypeId, SenderList>,
    id: usize,
}

impl EventMap {
    pub fn register<T: Clone + 'static>(&mut self) -> Receiver<T> {
        let (tx, rx) = channel::unbounded();
        self.inner
            .entry(TypeId::of::<T>())
            .or_default()
            .push((Id(self.id), Box::new(tx)));
        self.id += 1;
        rx
    }

    pub fn send<T: Clone + 'static>(&mut self, msg: T) {
        let mut bad = BTreeSet::new();
        if let Some(handlers) = self.get_senders::<T>() {
            for (id, handler) in handlers {
                match handler.send(msg.clone()) {
                    Err(TrySendError::Closed(_)) => {
                        // remove this id from the map
                        bad.insert(id);
                    }
                    Err(TrySendError::Full(_)) => unreachable!("unbounded channels cannot be full"),
                    Ok(..) => {}
                }
            }
        }
        self.remove::<T>(bad);
    }

    pub fn active<T: 'static>(&self) -> usize {
        self.inner
            .get(&TypeId::of::<T>())
            .map(Vec::len)
            .unwrap_or_default()
    }

    // TODO should this be public?
    pub fn get_senders<T: 'static>(&self) -> Option<Senders<'_, T>> {
        self.inner.get(&TypeId::of::<T>()).map(|list| Senders {
            inner: list.iter(),
            marker: PhantomData,
        })
    }

    // BTreeSet because it doesn't allocate if its empty, a HashSet will allocate like 4 pointers no matter what
    // a Vec with dedup might be more efficient, but that'd require some benchmarking
    //
    // but we use Set::remove(item) rather than Set::remove(index).
    // the Id isn't always the index into the vec
    pub(crate) fn remove<T: 'static>(&mut self, mut values: BTreeSet<Id>) {
        if values.is_empty() {
            // quick path because remove is called every dispatch
            return;
        }

        if let Some(inner) = self.inner.get_mut(&TypeId::of::<T>()) {
            // inverted so we remove them
            inner.retain(|(id, _)| !values.remove(&id))
        }
    }
}

pub struct Senders<'a, T: 'static> {
    inner: std::slice::Iter<'a, (Id, Box<dyn Any>)>,
    marker: PhantomData<T>,
}

impl<'a, T: 'static> Iterator for Senders<'a, T> {
    type Item = (Id, Sender<T>);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|(id, d)| {
            // TODO this should assert that it still exists
            let sender = d.downcast_ref::<Sender<T>>().cloned()?;
            Some((*id, sender))
        })
    }
}
