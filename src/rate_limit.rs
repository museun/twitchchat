// TODO actually write tests for this
#![allow(dead_code)]
/*!
A simple leaky-bucket style token-based rate limiter
*/

use std::time::{Duration, Instant};

/// A preset number of tokens as described by Twitch
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum RateClass {
    /// `20` per `30` seconds
    Regular,
    /// `100` per `30` seconds
    Moderator,
    /// `50` per `30` seconds
    Known,
    /// `7500` per `30` seconds
    Verified,
}

impl Default for RateClass {
    fn default() -> Self {
        Self::Regular
    }
}

impl RateClass {
    /// Number of tickets available for this class
    pub fn tickets(self) -> u64 {
        match self {
            Self::Regular => 20,
            Self::Moderator => 100,
            Self::Known => 50,
            Self::Verified => 7500,
        }
    }

    /// Period specified by Twitch
    pub const fn period() -> Duration {
        Duration::from_secs(30)
    }
}

/// A leaky-bucket style token-based rate limiter
#[derive(Debug, Clone)]
pub struct RateLimit {
    cap: u64,
    bucket: Bucket,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self::from_class(<_>::default())
    }
}

impl RateLimit {
    /// Overwrite the current capacity with this value
    pub fn set_cap(&mut self, cap: u64) {
        self.cap = cap
    }

    /// Overwrite the current period with this value
    pub fn set_period(&mut self, period: Duration) {
        self.bucket.period = period;
    }

    /// Get the current capacity with this value
    pub fn get_cap(&self) -> u64 {
        self.cap
    }

    /// Get the current period with this value
    pub fn get_period(&self) -> Duration {
        self.bucket.period
    }

    /// Create a rate limit from a RateClass
    pub fn from_class(rate_class: RateClass) -> Self {
        Self::full(rate_class.tickets(), RateClass::period())
    }

    /// Create a new rate limiter of `capacity` with an `initial` number of
    /// token and the `period` between refills
    pub fn new(cap: u64, initial: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::new(cap, initial, period),
        }
    }

    /// Create a new rate limiter that is pre-filled
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    pub fn full(cap: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::new(cap, cap, period),
        }
    }

    /// Create am empty rate limiter
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    ///
    /// This will block, at first, atleast one `period` until its filled
    pub fn empty(cap: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::new(cap, 0, period),
        }
    }

    /// Get the current available tokens
    pub fn get_available_tokens(&self) -> u64 {
        self.bucket.tokens
    }

    /// Tries to get the current RateClass.
    pub fn get_current_rate_class(&self) -> Option<RateClass> {
        const DUR: Duration = Duration::from_secs(30);

        let class = match (self.get_cap(), self.get_period()) {
            (20, DUR) => RateClass::Regular,
            (50, DUR) => RateClass::Known,
            (100, DUR) => RateClass::Moderator,
            (7500, DUR) => RateClass::Verified,
            _ => return None,
        };
        Some(class)
    }

    /// Consume a specific ammount of tokens
    ///
    /// # Returns    
    /// * Successful consumption (e.g. not blocking) will return how many tokens
    ///   are left
    /// * Failure to consume (e.g. out of tokens) will return a Duration of when
    ///   the bucket will be refilled
    pub fn consume(&mut self, tokens: u64) -> Result<u64, Duration> {
        let Self { bucket, .. } = self;

        let now = Instant::now();
        if let Some(n) = bucket.refill(now) {
            bucket.tokens = std::cmp::min(bucket.tokens + n, self.cap);
        }

        if tokens <= bucket.tokens {
            bucket.tokens -= tokens;
            bucket.backoff = 0;
            return Ok(bucket.tokens);
        }

        let prev = bucket.tokens;
        Err(bucket.estimate(tokens - prev, now))
    }
}

#[derive(Debug, Clone)]
struct Bucket {
    tokens: u64,
    backoff: u32,
    next: Instant,
    last: Instant,
    quantum: u64,
    period: Duration,
}

impl Bucket {
    fn new(tokens: u64, initial: u64, period: Duration) -> Self {
        let now = Instant::now();
        Self {
            tokens: initial,
            backoff: 0,
            next: now + period,
            last: now,
            quantum: tokens,
            period,
        }
    }

    fn refill(&mut self, now: Instant) -> Option<u64> {
        if now < self.next {
            return None;
        }

        let last = now.duration_since(self.last);
        let periods = last.as_nanos().checked_div(self.period.as_nanos())? as u64;
        self.last += self.period * (periods as u32);
        self.next = self.last + self.period;
        (periods * self.quantum).into()
    }

    fn estimate(&mut self, tokens: u64, now: Instant) -> Duration {
        let until = self.next.duration_since(now);
        let periods = (tokens.checked_add(self.quantum).unwrap() - 1) / self.quantum;
        until + self.period * (periods as u32 - 1)
    }
}
