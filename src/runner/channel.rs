use super::rate_limit::{PreviousRate, RateLimitedEncoder};
use crate::rate_limit::{RateClass, RateLimit};
use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

/// A channel that you are on.
///
/// This is exposed for 'advanced' users who want to modify the rate limiter.
///
/// # Warning
/// You shouldn't need to touch this unless you have a good reason to do so.
/// Improperly using this could result in Twitch disconnecting you, at best and
/// a ban at worst.
pub struct Channel {
    pub(crate) name: String,
    pub(crate) rate_limited: RateLimitedEncoder,
    pub(crate) previous: Option<PreviousRate>,
    pub(crate) rated_limited_at: Option<std::time::Instant>,
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel").field("name", &self.name).finish()
    }
}

impl Channel {
    pub(crate) fn new(name: String) -> Self {
        let rate_limit = RateLimit::from_class(RateClass::Regular);
        let rate_limited = RateLimitedEncoder {
            rate_limit,
            queue: VecDeque::new(),
        };
        Self {
            name,
            rate_limited,
            previous: None,
            rated_limited_at: None,
        }
    }

    /// Set the `RateClass` for this channel
    pub fn set_rate_class(&mut self, rate_class: RateClass) {
        self.rate_limited.rate_limit = RateLimit::from_class(rate_class);
        self.rated_limited_at.take();
    }

    /// Mark this channel as being under slow mode for `duration`
    pub fn enable_slow_mode(&mut self, duration: u64) {
        let rate = &mut self.rate_limited.rate_limit;
        self.previous.replace(PreviousRate {
            cap: rate.get_cap(),
            period: rate.get_period(),
        });

        rate.set_period(Duration::from_secs(duration))
    }

    /// Mark this channel as not being in slow mode
    pub fn disable_slow_mode(&mut self) {
        let PreviousRate { cap, period } = self.previous.take().unwrap_or_default();
        let rate = &mut self.rate_limited.rate_limit;
        rate.set_cap(cap);
        rate.set_period(period);
    }

    /// Mark that you've been rate limited on this channel
    pub fn set_rate_limited(&mut self) {
        self.rate_limited.rate_limit.set_cap(1);
        self.rated_limited_at.replace(std::time::Instant::now());
    }

    /// Reset to the default rate class
    pub fn reset_rate_limit(&mut self) {
        let PreviousRate { cap, period } = self.previous.take().unwrap_or_default();
        self.rate_limited.rate_limit = RateLimit::full(cap, period);
        self.rated_limited_at.take();
    }
}

#[derive(Debug, Default)]
pub struct Channels {
    pub map: HashMap<String, Channel>,
}

impl Channels {
    pub fn is_on(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Channel> {
        self.map.get_mut(name)
    }

    pub fn add(&mut self, name: &str) {
        // we already have this channel (there was a sync issue)
        if self.map.contains_key(name) {
            return;
        }

        let channel = Channel::new(name.to_string());
        self.map.insert(name.to_string(), channel);
    }

    pub fn remove(&mut self, name: &str) {
        self.map.remove(name);
    }
}
