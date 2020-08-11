use crate::{runner::ReadyMessage, DispatchError, IntoOwned, IrcMessage};
use std::collections::HashMap;

#[derive(Default)]
pub struct WaitFor {
    want: HashMap<&'static str, usize>,
    queue: HashMap<&'static str, IrcMessage<'static>>,
}

impl WaitFor {
    pub fn register<T>(&mut self)
    where
        T: ReadyMessage<'static>,
    {
        *self.want.entry(T::command()).or_default() += 1;
    }

    pub fn maybe_add<'a>(&mut self, msg: &IrcMessage<'a>) {
        if let Some((k, _)) = self.want.get_key_value(msg.get_command()) {
            self.queue.insert(*k, msg.into_owned());
        }
    }

    pub fn check_queue<T>(&mut self) -> Option<IrcMessage<'static>>
    where
        T: ReadyMessage<'static> + 'static + Send + Sync + Clone,
        DispatchError: From<T::Error>,
    {
        let msg = self.queue.remove(T::command())?;
        self.want.remove(T::command());
        Some(msg)
    }
}
