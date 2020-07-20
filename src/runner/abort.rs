pub type Abort = Notify;

#[derive(Debug, Default)]
pub(super) struct Notify {
    // TODO why isn't this wrapped in a clonable type?
    inner: event_listener::Event,
}

impl Notify {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Cancel this token
    pub(super) fn cancel(&self) {
        self.inner.notify(0);
    }

    pub(super) async fn wait_for(&self) {
        self.inner.listen().await
    }
}
