pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Determines whether this error is a blocking error
pub fn is_blocking_error(err: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    matches!(err.kind(), WouldBlock | Interrupted | TimedOut)
}

mod assert_configuration {
    #[cfg(all(feature = "ws", not(feature = "async")))]
    compile_error!("the `async` feature must be enabled when `ws` is enabled");
}
