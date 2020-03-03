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
