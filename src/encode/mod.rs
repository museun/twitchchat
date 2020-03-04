mod encoder;
pub use encoder::Encoder;

cfg_async! {
    mod async_encoder;
    #[doc(inline)]
    pub use async_encoder::AsyncEncoder;

    #[cfg(tests)]
    mod async_tests;
}

#[cfg(test)]
mod tests;

use crate::IntoChannel;
/// Checks a channel and turns it into a String
pub(self) fn conv_channel(ch: impl IntoChannel) -> std::io::Result<String> {
    use std::io::{Error, ErrorKind};
    ch.into_channel()
        .map(|s| s.to_string())
        .map_err(|err| Error::new(ErrorKind::Other, err))
}
