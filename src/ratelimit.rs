use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::MutexWrapper as Mutex;

/** RateLimit is a simple token bucket-style rate limiter

When its tokens are exhausted, it will block the current thread until its refilled

The limiter is cheaply-clonable
```no_run
# fn main() {
use std::time::Duration;
use twitchchat::helpers::RateLimit;

// limits to 3 `.take()` per 1 second
let limiter = RateLimit::full(3, Duration::from_secs(1));
for _ in 0..10 {
    // every 3 calls within 1 second will cause the next .take() to block
    // so this will take ~3 seconds to run (10 / 3 = ~3)
    limiter.take();
}

// initially empty, it will block for 1 second then block for every 3 calls per 1 second
let limiter = RateLimit::empty(3, Duration::from_secs(1));
for _ in 0..10 {
    limiter.take();
}
# }
```

The `_unsync()` variants create a cheaper, single-threaded form
**/
#[derive(Clone)]
pub struct RateLimit {
    cap: u64,
    bucket: Bucket,
}

impl RateLimit {
    /// Create a new, thread-safe limiter
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `initial` is how many are initially available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    pub fn new(cap: u64, initial: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::sync(cap, initial, period),
        }
    }

    /// Create a new, single-threaded limiter
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `initial` is how many are initially available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    pub fn new_unsync(cap: u64, initial: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::unsync(cap, initial, period),
        }
    }

    /// Create a thread-safe limiter thats pre-filled
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    #[inline]
    pub fn full(cap: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::sync(cap, cap, period),
        }
    }

    /// Create an empty thread-safe limiter
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    #[inline]
    pub fn empty(cap: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::sync(cap, 0, period),
        }
    }

    /// Returns the capacity of this RateLimit
    pub fn cap(&self) -> u64 {
        self.cap
    }

    /// Create a single-threaded limiter thats pre-filled
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    #[inline]
    pub fn full_unsync(cap: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::unsync(cap, cap, period),
        }
    }

    /// Create an empty single-threaded limiter
    ///
    /// `cap` is the number of total tokens available
    ///
    /// `period` is how long it'll take to refill all of the tokens
    #[inline]
    pub fn empty_unsync(cap: u64, period: Duration) -> Self {
        Self {
            cap,
            bucket: Bucket::unsync(cap, 0, period),
        }
    }

    /// Tries to consume `tokens`
    ///
    /// If it will consume more than available then an Error is returned.
    /// Otherwise it returns how many tokens are left
    ///
    /// This error is the `Duration` of the next available time
    pub fn consume(&self, tokens: u64) -> Result<u64, Duration> {
        let now = Instant::now();

        // no specialization yet and I don't want to have the consumers pass in
        // the {Sync|Unsync}Bucket, so simply just use a macro here
        macro_rules! consume {
            ($inner:expr) => {{
                if let Some(n) = $inner.refill(now) {
                    $inner.tokens = std::cmp::min($inner.tokens + n, self.cap);
                };

                if tokens <= $inner.tokens {
                    $inner.tokens -= tokens;
                    $inner.backoff = 0;
                    return Ok($inner.tokens);
                }

                let prev = $inner.tokens;
                Err($inner.estimate(tokens - prev, now))
            }};
        }

        match self.bucket {
            Bucket::Sync(ref sync) => {
                let mut sync = sync.lock();
                consume!(sync)
            }
            Bucket::Unsync(ref unsync) => {
                let mut unsync = unsync.borrow_mut();
                consume!(unsync)
            }
        }
    }

    /// Consumes `tokens` blocking if its trying to consume more than available
    ///
    /// Returns how many tokens are available
    pub fn throttle(&self, tokens: u64) -> u64 {
        loop {
            match self.consume(tokens) {
                Ok(rem) => return rem,
                Err(time) => std::thread::sleep(time),
            }
        }
    }

    /// Take a token, blocking if unavailable
    ///
    /// Returns how many tokens are available
    #[inline]
    pub fn take(&self) -> u64 {
        self.throttle(1)
    }
}

#[derive(Clone)]
enum Bucket {
    Sync(Arc<Mutex<Inner>>),
    Unsync(Rc<RefCell<Inner>>),
}

impl Bucket {
    fn sync(cap: u64, initial: u64, period: Duration) -> Self {
        Bucket::Sync(Arc::new(Mutex::new(Inner::new(cap, initial, period))))
    }
    fn unsync(cap: u64, initial: u64, period: Duration) -> Self {
        Bucket::Unsync(Rc::new(RefCell::new(Inner::new(cap, initial, period))))
    }
}

#[derive(Clone)]
struct Inner {
    tokens: u64,
    backoff: u32,

    next: Instant,
    last: Instant,

    quantum: u64,
    period: Duration,
}

impl Inner {
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
        Some(periods * self.quantum)
    }

    fn estimate(&mut self, tokens: u64, now: Instant) -> Duration {
        let until = self.next.duration_since(now);
        let periods = (tokens.checked_add(self.quantum).unwrap() - 1) / self.quantum;
        until + self.period * (periods as u32 - 1)
    }
}
