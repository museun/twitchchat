use futures_lite::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

type Receiver<T> = async_channel::Receiver<T>;

pin_project_lite::pin_project! {
    /// A [Stream] that produces an item
    ///
    /// The items are found [here]. The items wil be wrapped in an `Arc` and be `'static`.
    ///
    /// These are returned from an [event subscription][sub]
    ///
    /// [Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html
    /// [sub]: ./struct.Dispatcher.html#method.subscribe
    /// [here]: ./messages/index.html
    pub struct EventStream<T>{
        #[pin] pub(crate) inner: Receiver<T>,
    }
}

impl<T> std::fmt::Debug for EventStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStream")
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T: Clone> Stream for EventStream<T> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(ctx)
    }
}
