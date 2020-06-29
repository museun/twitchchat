//! Encode data to an [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) or [`tokio::io::AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html)

mod encoder;
pub use encoder::Encoder;

cfg_async! {
    mod async_encoder;
    #[doc(inline)]
    pub use async_encoder::AsyncEncoder;

    #[cfg(test)]
    mod async_tests;

    mod async_writer;
    pub use async_writer::{AsyncWriter, AsyncMpscWriter};
}

mod writer;
pub use writer::{SyncMpscWriter, SyncWriter};

#[cfg(test)]
mod tests;
