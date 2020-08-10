use async_channel::Receiver;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

pin_project_lite::pin_project! {
    /// A stream of events
    pub struct EventStream<T> {
        #[pin]
        pub(crate) inner: Receiver<T>,
    }
}

impl<T> std::fmt::Debug for EventStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventStream<{}>", std::any::type_name::<T>())
    }
}

impl<T> futures_lite::Stream for EventStream<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(ctx)
    }
}
