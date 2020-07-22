use super::Receiver;

use futures_lite::Stream;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pin_project_lite::pin_project! {
    pub struct EventStream<T> {
        #[pin]
        pub(crate) inner: Receiver<Arc<T>>,
    }
}

impl<T> Iterator for EventStream<T> {
    type Item = Arc<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.recv()
    }
}

impl<T> Stream for EventStream<T> {
    type Item = Arc<T>;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(ctx)
    }
}
