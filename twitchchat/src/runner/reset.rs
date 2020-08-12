use crate::{Receiver, Sender};

/// Configuration for what to do when retrying a connection
#[derive(Clone)]
pub struct ResetConfig {
    pub(crate) reset_handlers: Sender<()>,
}

impl std::fmt::Debug for ResetConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResetConfig").finish()
    }
}

impl ResetConfig {
    /// User Handlers (`EventStream`s) should be reset.
    ///
    /// When the connection is attempted again, it will clear any user `EventStreams` causing them to eventually produce None.
    ///
    /// This gives you a channel to 'wait' on so you can resubscribe.
    pub fn should_reset_handlers() -> (Self, Receiver<()>) {
        let (tx, rx) = crate::channel::bounded(1);
        let this = Self { reset_handlers: tx };
        (this, rx)
    }
}
