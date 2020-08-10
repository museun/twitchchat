use async_channel::Receiver;

/// An iterator of events
pub struct EventIter<T> {
    pub(crate) inner: Receiver<T>,
}

impl<T> std::fmt::Debug for EventIter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventIter<{}>", std::any::type_name::<T>())
    }
}

impl<T> Iterator for EventIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.try_recv().ok()
    }
}
