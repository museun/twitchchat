pub(crate) fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub(crate) fn is_blocking_error(err: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    matches!(err.kind(), WouldBlock | Interrupted | TimedOut)
}
