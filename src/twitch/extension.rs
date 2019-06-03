use super::{Error, Writer};
use ratelimit::RateLimit;

/// Writer extensions
pub trait WriterExt {
    /// Join a (huge) list of channels
    ///
    /// This will efficiently partition all of the JOIN commands into max-sized
    /// messages
    ///
    /// Ensuring the channel names are properly formatted and doing the least
    /// amount of actual writes as possible
    ///
    /// Returns a [`JoinStats`](./struct.JoinStats.html) which details how many messages were sent and the total number of channels
    ///
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// use twitchchat::WriterExt as _;
    /// let writer = client.writer();
    /// writer
    ///     .join_many(
    ///         std::fs::read_to_string("active.txt")
    ///             .unwrap()
    ///             .split('\n')
    ///             .map(str::trim),
    ///     )
    ///     .unwrap();
    /// ```
    fn join_many<'a, I>(&self, channels: I) -> Result<JoinStats, Error>
    where
        I: IntoIterator + 'a,
        I::Item: AsRef<str> + 'a;

    /// Join a (huge) list of channels but using a [`RateLimit`](./helpers/struct.RateLimit.html)
    ///
    /// Same as [`WriterExt::join_many`](./trait.WriterExt.html#method.join_many), but takes in an optional RateLimit
    ///
    /// If no rate limiter is provided then a default is used (50 channels per 15 seconds)
    ///
    /// Returns a [`JoinStats`](./struct.JoinStats.html) which details how many messages were sent and the total number of channels
    fn join_many_limited<'a, I>(
        &self,
        channels: I,
        rate: Option<RateLimit>,
    ) -> Result<JoinStats, Error>
    where
        I: IntoIterator + 'a,
        I::Item: AsRef<str> + 'a;
}

impl WriterExt for Writer {
    fn join_many_limited<'a, I>(
        &self,
        channels: I,
        rate: Option<RateLimit>,
    ) -> Result<JoinStats, Error>
    where
        I: IntoIterator + 'a,
        I::Item: AsRef<str> + 'a,
    {
        let state = JoinState::Try(rate);
        join_limited(self, channels, state)
    }

    fn join_many<'a, I>(&self, channels: I) -> Result<JoinStats, Error>
    where
        I: IntoIterator + 'a,
        I::Item: AsRef<str> + 'a,
    {
        let state = JoinState::None;
        join_limited(self, channels, state)
    }
}

enum JoinState {
    Try(Option<RateLimit>),
    None,
}

fn join_limited<'a, I>(w: &Writer, iter: I, state: JoinState) -> Result<JoinStats, Error>
where
    I: IntoIterator + 'a,
    I::Item: AsRef<str> + 'a,
{
    let mut rate = match state {
        JoinState::Try(rate) => Some(
            rate.unwrap_or_else(|| RateLimit::full_unsync(50, std::time::Duration::from_secs(15))),
        ),
        _ => None,
    };

    let mut buf = String::with_capacity(512);
    let mut stats = JoinStats {
        channels: 0,
        messages: 0,
    };

    for channel in iter.into_iter() {
        let channel = channel.as_ref();
        let len = if channel.starts_with('#') {
            channel.len()
        } else {
            channel.len() + 1
        };

        if buf.len() + len + 1 > 510 {
            w.write_line(&buf)?;
            buf.clear();

            if let Some(ref mut rate) = &mut rate {
                let _ = rate.take();
            }

            stats.messages += 1;
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
        stats.channels += 1;
    }

    if !buf.is_empty() {
        w.write_line(&buf)?;
        stats.messages += 1;
    }

    Ok(stats)
}

/// The join extensions return this so some stats are available
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct JoinStats {
    /// How many channels were attempted to be joined
    pub channels: usize,
    /// How many messages were sent to the server
    pub messages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn make_channel_list() -> impl Iterator<Item = String> {
        use rand::prelude::*;
        std::iter::from_fn(move || {
            let mut rng = thread_rng();
            let range = rng.gen_range(5, 30);
            Some(
                rng.sample_iter(&rand::distributions::Alphanumeric)
                    .take(range)
                    .collect(),
            )
        })
    }

    #[test]
    fn join_many() {
        let mut stream = helpers::TestStream::new();
        let (r, w) = sync_adapters(stream.clone(), stream.clone());
        let client = Client::new(r, w);

        let _ = client
            .writer()
            .join_many(make_channel_list().take(1000))
            .unwrap();

        let line = stream.drain_buffer().unwrap();

        for line in line.split_terminator("\r\n") {
            let len = line.len();
            assert!(len <= 510, "{} <= 510", len);
        }
    }

    #[test]
    fn join_many_limited() {
        let mut stream = helpers::TestStream::new();
        let (r, w) = sync_adapters(stream.clone(), stream.clone());
        let client = Client::new(r, w);

        let start = std::time::Instant::now();

        let _ = client
            .writer()
            .join_many_limited(
                make_channel_list().take(1000),
                Some(RateLimit::full_unsync(
                    5,
                    std::time::Duration::from_millis(10),
                )),
            )
            .unwrap();

        let line = stream.drain_buffer().unwrap();
        for line in line.split_terminator("\r\n") {
            let len = line.len();
            assert!(len <= 510, "{} <= 510", len);
        }

        let end = std::time::Instant::now();
        assert!(end - start > std::time::Duration::from_millis(50));
    }
}
