use super::{Client, Error};
use crate::helpers::RateLimit;
use std::io::Write;

/// Client extensions
pub trait ClientExt {
    /// Join a (huge) list of channels
    ///
    /// This will efficiently partition all of the JOIN commands into max-sized
    /// messages
    ///
    /// Ensuring the channel names are properly formatted and doing the least
    /// amount of actual writes as possible
    ///
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client, ClientExt};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// client
    ///     .join_many(
    ///         std::fs::read_to_string("active.txt")
    ///             .unwrap()
    ///             .split('\n')
    ///             .map(str::trim),
    ///     )
    ///     .unwrap();
    /// ```
    fn join_many<'a, I, S>(&mut self, channels: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = S> + 'a,
        S: AsRef<str> + 'a;

    /// Join a (huge) list of channels but using a [`RateLimit`](./helpers/struct.RateLimit.html)
    ///
    /// Same as [`ClientExt::join_many`](./trait.ClientExt.html#method.join_many), but takes in an optional RateLimit
    ///
    /// If no rate limiter is provided then a default is used (50 channels per 15 seconds)
    fn join_many_limited<'a, I, S>(
        &mut self,
        channels: I,
        rate: Option<RateLimit>,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = S> + 'a,
        S: AsRef<str> + 'a;
}

impl<R, W: Write> ClientExt for Client<R, W> {
    fn join_many_limited<'a, I, S>(
        &mut self,
        channels: I,
        rate: Option<RateLimit>,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = S> + 'a,
        S: AsRef<str> + 'a,
    {
        self.join_limited(channels, true, rate)
    }

    fn join_many<'a, I, S>(&mut self, channels: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = S> + 'a,
        S: AsRef<str> + 'a,
    {
        self.join_limited(channels, false, None)
    }
}

impl<R, W: Write> Client<R, W> {
    fn join_limited<'a, I, S>(
        &mut self,
        channels: I,
        try_rate: bool,
        rate: Option<RateLimit>,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = S> + 'a,
        S: AsRef<str> + 'a,
    {
        use std::time::Duration;

        let mut rate = if try_rate {
            Some(rate.unwrap_or_else(|| RateLimit::full_unsync(50, Duration::from_secs(15))))
        } else {
            None
        };

        let w = self.writer();

        let mut buf = String::with_capacity(512);

        let mut count = 0;
        let mut prev = 0;
        for channel in channels.into_iter() {
            let channel = channel.as_ref();
            if buf.len() + channel.len() + 1 > 510
                || Some(count) == rate.as_ref().map(RateLimit::cap)
            {
                // TODO have writer return a MutexGuard
                w.write_line(&buf)?;
                buf.clear();

                if let Some(ref mut rate) = &mut rate {
                    for _ in 0..if prev != 0 { prev } else { count } {
                        let _ = rate.take();
                    }
                }

                if let Some(cap) = rate.as_ref().map(RateLimit::cap) {
                    prev = 0;
                    if count == cap {
                        count = 0
                    } else {
                        prev = cap - prev
                    }
                }
            }

            if buf.is_empty() {
                buf.push_str("JOIN ");
            } else {
                buf.push(',');
            }

            if !channel.starts_with('#') {
                buf.push_str(&["#", channel].concat());
            } else {
                buf.push_str(&channel);
            }
            count += 1;
        }

        if !buf.is_empty() {
            w.write_line(&buf)?;
        }

        Ok(())
    }
}
