use crate::{Receiver, Sender};

#[derive(Clone)]
pub struct ResetConfig {
    pub(crate) reset_handlers: Sender<()>,
}

impl ResetConfig {
    pub fn should_reset_handlers() -> (Self, Receiver<()>) {
        let (tx, rx) = crate::channel::bounded(1);
        let this = Self { reset_handlers: tx };
        (this, rx)
    }
}
