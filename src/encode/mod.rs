use crate::IntoChannel;

/// Checks a channel and turns it into a String
pub(self) fn conv_channel(ch: impl IntoChannel) -> Result<String, crate::Error> {
    ch.into_channel().map(|s| s.to_string()).map_err(Into::into)
}

mod encoder;
pub use encoder::Encoder;

cfg_async! {
    mod async_encoder;
    #[doc(inline)]
    pub use async_encoder::AsyncEncoder;

    #[cfg(test)]
    mod async_tests;
}

#[cfg(test)]
mod tests;
