use async_channel::{TryRecvError, TrySendError};
use futures_lite::Stream;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct Sender<T> {
    inner: async_channel::Sender<T>,
    unparker: parking::Unparker,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        Self {
            inner: self.inner.clone(),
            unparker: self.unparker.clone(),
        }
    }
}

impl<T> Sender<T> {
    const fn new(inner: async_channel::Sender<T>, unparker: parking::Unparker) -> Self {
        Self { inner, unparker }
    }

    pub fn send(&self, item: T) -> Result<(), TrySendError<T>> {
        self.unparker.unpark();
        self.inner.try_send(item)
    }

    pub async fn send_async(&self, item: T) -> bool {
        self.unparker.unpark();
        self.inner.send(item).await.is_ok()
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug, Clone)]
    pub struct Receiver<T> {
        #[pin]
        inner: async_channel::Receiver<T>,
        parker: Arc<parking::Parker>, // this is !Sync
    }
}

impl<T> Receiver<T> {
    pub fn new(inner: async_channel::Receiver<T>, parker: parking::Parker) -> Self {
        Self {
            inner,
            parker: Arc::new(parker),
        }
    }

    pub fn recv(&self) -> Option<T> {
        loop {
            self.parker.park();

            match self.inner.try_recv() {
                Ok(d) => return Some(d),
                Err(TryRecvError::Closed) => return None,

                // someone already grabbed the element
                Err(TryRecvError::Empty) => continue,
            }
        }
    }

    pub async fn recv_async(&self) -> Option<T> {
        self.inner.recv().await.ok()
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(ctx)
    }
}

pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let (parker, unparker) = parking::pair();
    let (tx, rx) = async_channel::bounded(cap);
    (Sender::new(tx, unparker), Receiver::new(rx, parker))
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let (parker, unparker) = parking::pair();
    let (tx, rx) = async_channel::unbounded();
    (Sender::new(tx, unparker), Receiver::new(rx, parker))
}
