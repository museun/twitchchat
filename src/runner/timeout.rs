use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug)]
pub enum TimeoutState {
    WaitingForPong(Instant),
    Activity(Instant),
    Start,
}

impl TimeoutState {
    pub fn activity() -> Self {
        Self::Activity(Instant::now())
    }

    pub fn waiting_for_pong() -> Self {
        Self::WaitingForPong(Instant::now())
    }
}

pub const WINDOW: Duration = Duration::from_secs(45);
pub const TIMEOUT: Duration = Duration::from_secs(10);
pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(30);

pub async fn next_delay() {
    futures_timer::Delay::new(WINDOW).await
}
