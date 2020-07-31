use super::{channel, Receiver, Sender};

use async_channel::TrySendError;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
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
        let mut bad = HashSet::new();
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

    pub fn get<T: 'static>(&self) -> Option<impl Iterator<Item = (Id, Sender<T>)> + '_> {
        // TODO debug assert our lengths are the same
        self.inner.get(&TypeId::of::<T>()).map(|list| {
            list.iter()
                .flat_map(|(id, d)| d.downcast_ref::<Sender<T>>().cloned().map(|t| (*id, t)))
        })
    }

    pub(crate) fn remove<T: 'static>(&mut self, mut values: HashSet<Id>) {
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
