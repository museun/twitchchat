use super::*;
use tokio::stream::Stream;

/// A [Stream] that produces an item
///
/// The items are found [here]. The items wil be wrapped in an `Arc` and be `'static`.
///
/// These are returned from an [event subscription][sub]
///
/// [Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html
/// [sub]: ./struct.Dispatcher.html#method.subscribe
/// [here]: ./messages/index.html
pub struct EventStream<T>(pub(crate) mpsc::UnboundedReceiver<T>);

impl<T> std::fmt::Debug for EventStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStream")
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T: Clone> Stream for EventStream<T> {
    type Item = T;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_recv(cx)
    }
}
