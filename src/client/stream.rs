use super::*;

/// A [Stream] that produces an item
///
/// These are returned from an [event subscription][sub]
///
/// [Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
/// [sub]: ./struct.Dispatcher.html#method.subscribe
pub struct EventStream<T>(pub(super) mpsc::UnboundedReceiver<T>);

impl<T> Stream for EventStream<T>
where
    T: Clone,
{
    type Item = T;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_recv(cx)
    }
}
