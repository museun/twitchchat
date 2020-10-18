//! Simple async/sync channels used in various parts of this crate.
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// An error on send
#[derive(Debug)]
pub enum TrySendError<T> {
    /// The receiver was closed
    Closed(T),
    /// The receiver was full
    Full(T),
}

/// Async and Sync MPMP Sender.
#[derive(Clone)]
pub struct Sender<T> {
    inner: async_channel::Sender<T>,
}

impl<T> std::fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender").finish()
    }
}

impl<T> Sender<T> {
    /// Send this item asynchronously.
    ///
    /// On failure, return the sent item.
    pub async fn send(&self, item: T) -> Result<(), T>
    where
        T: Send,
    {
        self.inner.send(item).await.map_err(|e| e.into_inner())
    }

    /// Send this item synchronously.
    ///
    /// On failure, returns why and the item.
    pub fn try_send(&self, item: T) -> Result<(), TrySendError<T>> {
        self.inner.try_send(item).map_err(|e| match e {
            async_channel::TrySendError::Full(t) => TrySendError::Full(t),
            async_channel::TrySendError::Closed(t) => TrySendError::Closed(t),
        })
    }
}

pin_project_lite::pin_project! {
    /// Async and Sync MPMP Receiver.
    #[derive(Clone)]
    pub struct Receiver<T> {
        #[pin]
        inner: async_channel::Receiver<T>,
    }
}

impl<T> Receiver<T> {
    /// Asynchronously receives an item
    ///
    /// If this returns None, the Sender was closed
    pub async fn recv(&self) -> Option<T>
    where
        T: Send,
    {
        self.inner.recv().await.ok()
    }

    /// Close the receiver
    pub fn close(&self) -> bool {
        self.inner.close()
    }

    /// Synchronously receives an item
    ///
    /// If this returns None, the Sender was closed
    pub fn try_recv(&self) -> Option<T> {
        self.inner.try_recv().ok()
    }
}

impl<T> futures_lite::Stream for Receiver<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(ctx)
    }
}

/// Create a bounded channel
pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = async_channel::bounded(cap);
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create an unbounded channel
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = async_channel::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}
