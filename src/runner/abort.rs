use std::sync::Arc;
use tokio::sync::Notify;

/// An Abort token that can be used for stopping the client early
#[derive(Clone, Debug, Default)]
pub(super) struct Abort {
    inner: Arc<Notify>,
}

impl Abort {
    /// Cancel this token
    pub(super) fn cancel(&self) {
        self.inner.notify();
    }

    pub(super) async fn wait_for(&self) {
        self.inner.notified().await;
    }
}
