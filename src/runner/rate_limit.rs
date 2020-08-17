use crate::{rate_limit::RateLimit, RateClass};
use futures_lite::{AsyncWrite, AsyncWriteExt};
use std::{collections::VecDeque, time::Duration};

pub struct RateLimitedEncoder {
    pub(crate) rate_limit: RateLimit,
    pub(crate) queue: VecDeque<Box<[u8]>>,
}

impl RateLimitedEncoder {
    pub async fn drain_until_blocked<W>(
        &mut self,
        name: &str,
        limit: &mut u64,
        sink: &mut W,
    ) -> std::io::Result<()>
    where
        W: AsyncWrite + Send + Sync + Unpin + ?Sized,
    {
        log::error!("'{}' has {} queued messages", name, self.queue.len());

        while let Some(data) = self.queue.pop_front() {
            match self.rate_limit.consume(1) {
                Ok(..) => {
                    *limit = limit.saturating_sub(1);
                    log::trace!(
                        target: "twitchchat::encoder",
                        "> {}",
                        std::str::from_utf8(&*data).unwrap().escape_debug()
                    );
                    sink.write_all(&*data).await?;
                }
                Err(..) => {
                    log::warn!(
                        target: "twitchchat::rate_limit",
                        "local rate limit for '{}' hit",
                        name
                    );
                    break;
                }
            }
            if *limit == 0 {
                break;
            }
        }

        Ok(())
    }

    pub fn enqueue(&mut self, msg: Box<[u8]>) {
        self.queue.push_back(msg);
    }
}

pub struct PreviousRate {
    pub cap: u64,
    pub period: Duration,
}

impl Default for PreviousRate {
    fn default() -> Self {
        Self {
            cap: RateClass::Regular.tickets(),
            period: RateClass::period(),
        }
    }
}
