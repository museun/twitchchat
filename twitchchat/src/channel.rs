use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub enum TrySendError<T> {
    Closed(T),
    Full(T),
}

#[derive(Clone)]
pub struct Sender<T> {
    inner: async_channel::Sender<T>,
}

impl<T> Sender<T> {
    pub async fn send(&self, item: T) -> Result<(), T>
    where
        T: Send,
    {
        self.inner.send(item).await.map_err(|e| e.into_inner())
    }

    pub fn try_send(&self, item: T) -> Result<(), TrySendError<T>> {
        self.inner.try_send(item).map_err(|e| match e {
            async_channel::TrySendError::Full(t) => TrySendError::Full(t),
            async_channel::TrySendError::Closed(t) => TrySendError::Closed(t),
        })
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct Receiver<T> {
        #[pin]
        inner: async_channel::Receiver<T>,
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&self) -> Option<T>
    where
        T: Send,
    {
        self.inner.recv().await.ok()
    }

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

pub(crate) fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = async_channel::bounded(cap);
    (Sender { inner: tx }, Receiver { inner: rx })
}

#[allow(dead_code)]
pub(crate) fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = async_channel::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}
