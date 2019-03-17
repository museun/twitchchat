use std::sync::mpsc;
use std::time::Duration;

/// A basic rate limiter backed by an mpsc sync channel
///
/// Its very spikey, but thats a future-me problem
/// It also spawns a thread for each instance
pub struct RateLimit {
    tx: mpsc::SyncSender<()>,
    quantum: Duration,
    cap: usize,
}

impl RateLimit {
    /// Create a new rate limiter that has `cap` tokens per `quantum` seconds
    pub fn new(cap: usize, quantum: u64) -> Self {
        debug_assert!(cap > 0);
        debug_assert!(quantum > 0);
        let (tx, rx) = mpsc::sync_channel::<()>(cap - 1);

        let quantum = Duration::from_secs(quantum);
        let _ = std::thread::spawn(move || 'outer: loop {
            loop {
                match rx.try_recv() {
                    Ok(()) => {}
                    Err(mpsc::TryRecvError::Empty) => break,
                    _ => break 'outer,
                }
            }
            std::thread::sleep(quantum);
        });

        Self { tx, quantum, cap }
    }

    /// Returns the capacity of the rate limit (e.g. how many before it blocks)
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Take a token
    pub fn take(&mut self) {
        use mpsc::TrySendError;
        match self.tx.try_send(()) {
            Ok(()) => return,
            Err(TrySendError::Full(..)) => {}
            Err(TrySendError::Disconnected(..)) => unreachable!(),
        };
        std::thread::sleep(self.quantum);
    }
}
