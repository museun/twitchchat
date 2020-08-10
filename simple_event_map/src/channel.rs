#![allow(clippy::redundant_pub_crate)]

/// A sender that lets you send messages to an EventStream
pub struct Sender<T>(async_channel::Sender<T>);

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Sender<T> {
    /// Send this message. This will not block.
    ///
    /// This returns an Error with the Item if the EventStream was closed
    pub fn send(&self, item: T) -> Result<(), T> {
        self.0.try_send(item).map_err(|err| match err {
            async_channel::TrySendError::Full(..) => unreachable!(),
            async_channel::TrySendError::Closed(item) => item,
        })
    }
}

pub(crate) fn unbounded<T>() -> (Sender<T>, async_channel::Receiver<T>) {
    let (tx, rx) = async_channel::unbounded();
    (Sender(tx), rx)
}
