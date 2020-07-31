use super::{channel, Receiver, Sender};

use async_channel::TrySendError;
use std::{
    any::{Any, TypeId},
    collections::{BTreeSet, HashMap},
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
        if let Some(handlers) = self.get::<T>() {
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
    pub fn get<T: 'static>(&self) -> Option<impl Iterator<Item = (Id, Sender<T>)> + '_> {
        let list = self.inner.get(&TypeId::of::<T>())?;

        let iter = list.iter().flat_map(|(id, d)| {
            // TODO this should assert that it still exists
            let sender = d.downcast_ref().cloned()?;
            Some((*id, sender))
        });

        Some(iter)
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
