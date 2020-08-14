use crate::{runner::ReadyMessage, Capability, DispatchError, IntoOwned, IrcMessage};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct WaitFor {
    want: HashMap<&'static str, usize>,
    queue: HashMap<&'static str, IrcMessage<'static>>,

    pub are_we_anonymous: bool,
    seen_ready: bool,
    seen_caps: HashSet<Capability>,
}

impl WaitFor {
    pub fn register<T>(&mut self) -> Result<(), crate::RunnerError>
    where
        T: ReadyMessage<'static>,
    {
        if self.are_we_anonymous && T::required_cap().is_some() {
            return Err(crate::RunnerError::RequiredCaps(
                T::required_cap().unwrap(),
                crate::util::trim_type_name::<T>(),
            ));
        }

        *self.want.entry(T::command()).or_default() += 1;
        Ok(())
    }

    pub fn maybe_add<'a>(&mut self, msg: &IrcMessage<'a>) {
        if let IrcMessage::CAP = msg.get_command() {
            use crate::Validator as _;

            if let Ok("ACK") = msg.expect_arg(1) {
                if let Some(cap) = msg.expect_data().ok().and_then(Capability::maybe_from_str) {
                    self.seen_caps.insert(cap);
                }
            }
        }

        if let IrcMessage::READY = msg.get_command() {
            self.seen_ready = true;
        }

        if let Some((k, _)) = self.want.get_key_value(msg.get_command()) {
            self.queue.insert(*k, msg.into_owned());
        }
    }

    pub fn check_queue<T>(&mut self) -> CapsStatus<IrcMessage<'static>>
    where
        T: ReadyMessage<'static> + 'static,
        DispatchError: From<T::Error>,
    {
        if self.seen_ready {
            if let Some(cap) = T::required_cap() {
                if !self.seen_caps.contains(&cap) {
                    return CapsStatus::RequiredCap(cap);
                }
            }
        }

        let msg = match self.queue.remove(T::command()) {
            Some(msg) => msg,
            None => return CapsStatus::NotSeen,
        };

        self.want.remove(T::command());

        CapsStatus::Seen(msg)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CapsStatus<T> {
    RequiredCap(Capability),
    Seen(T),
    NotSeen,
}
