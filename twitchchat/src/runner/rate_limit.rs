use crate::{rate_limit::RateLimit, AsyncEncoder, Encodable, RateClass};
use futures_lite::AsyncWrite;
use std::collections::{HashMap, VecDeque};

pub enum RateStatus {
    Limited,
    SlowMode(u64),
    Normal,
}

pub struct Channel {
    status: RateStatus,
    queue: RateLimitQueue<Box<[u8]>>,
    previous: Option<RateClass>,
}

impl Channel {
    pub fn new(rate_limit: RateLimit) -> Self {
        Self {
            status: RateStatus::Normal,
            queue: RateLimitQueue::new(rate_limit),
            previous: None,
        }
    }

    pub fn change_rate_class(&mut self, rate_class: RateClass) {
        // NOTE: this will reset the rate limiter, which users can 'abuse' (i.e. get banned by Twitch)
        // this isn't something I'm going to detect because its not my problem.
        let _ = std::mem::replace(
            &mut self.queue.rate_limit,
            RateLimit::from_class(rate_class),
        );
    }

    pub async fn enqueue<W>(
        &mut self,
        data: Box<[u8]>,
        enc: &mut AsyncEncoder<W>,
    ) -> std::io::Result<()>
    where
        W: AsyncWrite + Send + Sync + Unpin,
    {
        self.queue.enqueue(data, enc).await
    }

    pub fn set_status(&mut self, status: RateStatus) -> RateStatus {
        match self.status {
            // if we're limited, use the current cap but reduce the tokens down to 1
            RateStatus::Limited => {
                if self.previous.is_none() {
                    self.previous.replace(
                        self.queue
                            .rate_limit
                            .get_current_rate_class()
                            .unwrap_or_default(),
                    );
                }
                self.queue.rate_limit.set_cap(1);
            }
            // if we're in slow mode, set the cap to 1 and the duration to 1
            RateStatus::SlowMode(dur) => {
                if self.previous.is_none() {
                    self.previous.replace(
                        self.queue
                            .rate_limit
                            .get_current_rate_class()
                            .unwrap_or_default(),
                    );
                }

                self.queue.rate_limit.set_cap(1);
                self.queue
                    .rate_limit
                    .set_period(std::time::Duration::from_secs(dur));
            }
            // otherwise reset the rate limiter if we were previously rate limited
            RateStatus::Normal => {
                let class = self.previous.take().unwrap_or_default();
                self.change_rate_class(class);
            }
        }

        // and update our status
        std::mem::replace(&mut self.status, status)
    }
}

pub struct RateLimitedChannels {
    map: HashMap<String, Channel>,
    global: RateLimitQueue<Box<[u8]>>,
}

impl RateLimitedChannels {
    pub fn new(global_rate: RateLimit) -> Self {
        Self {
            map: HashMap::default(),
            global: RateLimitQueue::new(global_rate),
        }
    }

    pub fn set_global_rate_limit(&mut self, rate_class: RateClass) {
        self.global.rate_limit = RateLimit::from_class(rate_class);
    }

    pub async fn enqueue_for<W>(
        &mut self,
        channel: String,
        data: Box<[u8]>,
        enc: &mut AsyncEncoder<W>,
    ) -> std::io::Result<()>
    where
        W: AsyncWrite + Send + Sync + Unpin,
    {
        // TODO use a better error than an io::Error for this
        let ch = self.get_channel(&channel).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("you're not on '{}'", channel),
            )
        })?;
        ch.enqueue(data, enc).await
    }

    pub async fn global_enqueue<W>(
        &mut self,
        data: Box<[u8]>,
        enc: &mut AsyncEncoder<W>,
    ) -> std::io::Result<()>
    where
        W: AsyncWrite + Send + Sync + Unpin,
    {
        self.global.enqueue(data, enc).await
    }

    pub fn get_channel(&mut self, channel: &str) -> Option<&mut Channel> {
        self.map.get_mut(channel)
    }

    pub fn add_channel(&mut self, channel: &str, rate_limit: RateLimit) {
        self.map
            .entry(channel.to_string())
            .or_insert_with(|| Channel::new(rate_limit));
    }

    pub fn remove_channel(&mut self, channel: &str) {
        self.map.remove(channel);
    }

    pub fn enable_slow(&mut self, channel: &str) {
        if let Some(ch) = self.map.get_mut(channel) {
            ch.set_status(RateStatus::SlowMode(30));
        }
    }

    pub fn disable_slow(&mut self, channel: &str) {
        if let Some(ch) = self.map.get_mut(channel) {
            ch.set_status(RateStatus::Normal);
        }
    }

    pub fn rate_limited_on_channel(&mut self, channel: &str) {
        if let Some(ch) = self.map.get_mut(channel) {
            ch.set_status(RateStatus::Limited);
        }
    }

    pub fn set_slow_duration(&mut self, channel: &str, dur: u64) {
        if let Some(ch) = self.map.get_mut(channel) {
            ch.set_status(RateStatus::SlowMode(dur));
        }
    }
}

struct RateLimitQueue<T> {
    queue: VecDeque<T>,
    rate_limit: RateLimit,
}

impl<T> RateLimitQueue<T> {
    pub fn new(rate_limit: RateLimit) -> Self {
        Self {
            queue: VecDeque::default(),
            rate_limit,
        }
    }

    pub async fn enqueue<W>(&mut self, msg: T, enc: &mut AsyncEncoder<W>) -> std::io::Result<()>
    where
        W: AsyncWrite + Send + Sync + Unpin,
        T: Encodable + Send + Sync,
    {
        match self.rate_limit.consume(1) {
            // we don't have any messages queued so lets send the current one
            Ok(_) if self.queue.is_empty() => {
                enc.encode(msg).await?;
            }
            // otherwise drain as much of the queue as possible
            Ok(tokens) => {
                log::trace!(target: "twitchchat::rate_limit", "we're draining the queue for {} items", tokens);
                // we have 'tokens' available. so write up to that many messages
                let max = std::cmp::min(tokens as usize, self.queue.len());
                for msg in self.queue.drain(0..max) {
                    enc.encode(msg).await?;
                }

                log::trace!(target: "twitchchat::rate_limit", "enqueued new message");
                self.queue.push_back(msg);
            }
            // we're limited, so enqueue the message
            Err(dur) => {
                log::trace!(target: "twitchchat::rate_limit", "we're limited for: {:?}. enqueuing message", dur);
                self.queue.push_back(msg)
            }
        }

        Ok(())
    }
}
