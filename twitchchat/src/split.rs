//! This module lets you split an `io-object` into `read`/`write` halves, wrapped in an `Decoder`/`Encoder`
//!
//! This module is split into two parts: [`sync`] and [`async`]
//!
//! ## sync
//!
//! | function                             | output                                                                                |
//! | ------------------------------------ | ------------------------------------------------------------------------------------- |
//! | [`make_pair`][make_pair]             | a [`Decoder`][decoder], [`Encoder`][encoder] pair                                     |
//! | [`make_boxed_pair`][make_boxed_pair] | a typed erased [`BoxedDecoder`][boxed_decoder], [`BoxedEncoder`][boxed_encoder] pair. |
//!
//! ## async
//!
//! | function                                   | output                                                                                            |
//! | ------------------------------------------ | ------------------------------------------------------------------------------------------------- |
//! | [`make_pair`][async_make_pair]             | a [`AsyncDecoder`][async_decoder], [`AsyncEncoder`][async_encoder] pair                           |
//! | [`make_boxed_pair`][async_make_boxed_pair] | a typed erased [`BoxedDecoder`][async_boxed_decoder], [`BoxedEncoder`][async_boxed_encoder] pair. |
//!
//! [make_pair]: crate::split::sync::make_pair
//! [make_boxed_pair]: crate::split::sync::make_boxed_pair
//! [async_make_pair]: crate::split::async::make_pair
//! [async_make_boxed_pair]: crate::split::async::make_boxed_pair
//! [decoder]: crate::Decoder
//! [encoder]: crate::Encoder
//! [boxed_decoder]: crate::split::sync::BoxedDecoder
//! [boxed_encoder]: crate::split::sync::BoxedEncoder
//! [async_decoder]: crate::AsyncDecoder
//! [async_encoder]: crate::AsyncEncoder
//! [async_boxed_decoder]: crate::split::async::BoxedDecoder
//! [async_boxed_encoder]: crate::split::async::BoxedEncoder
//!

cfg_async! {
/// Asynchronous helpers
pub mod r#async {
    use crate::{AsyncDecoder, AsyncEncoder};
    /// A boxed [`futures_io::AsyncRead`][read] trait object
    ///
    /// [read]: https://docs.rs/futures-io/0.3.6/futures_io/trait.AsyncRead.html
    pub type BoxedRead = Box<dyn futures_lite::AsyncRead + Send + Unpin>;
    /// A boxed [`futures_io::AsyncWrite`][write] trait object
    ///
    /// [write]: https://docs.rs/futures-io/0.3.6/futures_io/trait.AsyncWrite.html
    pub type BoxedWrite = Box<dyn futures_lite::AsyncWrite + Send + Unpin>;

    /// A boxed [`AsyncDecoder`]
    pub type BoxedDecoder = AsyncDecoder<BoxedRead>;

    /// A boxed [`AsyncEncoder`]
    pub type BoxedEncoder = AsyncEncoder<BoxedWrite>;

    /// Split an Async IO object and return [`AsyncDecoder`]/[`AsyncEncoder`]
    pub fn make_pair<IO>(
        io: IO,
    ) -> (
        AsyncDecoder<futures_lite::io::ReadHalf<IO>>,
        AsyncEncoder<futures_lite::io::WriteHalf<IO>>,
    )
    where
        IO: futures_lite::AsyncRead + futures_lite::AsyncWrite,
        IO: Send + Unpin,
    {
        let (read, write) = futures_lite::io::split(io);
        (AsyncDecoder::new(read), AsyncEncoder::new(write))
    }
    /// Split an Async IO object and return type-erased [`AsyncDecoder`]/[`AsyncEncoder`]
    pub fn make_boxed_pair<IO>(io: IO) -> (BoxedDecoder, BoxedEncoder)
    where
        IO: futures_lite::AsyncRead + futures_lite::AsyncWrite,
        IO: Send + Unpin + 'static,
    {
        let (read, write) = futures_lite::io::split(io);
        let read: BoxedRead = Box::new(read);
        let write: BoxedWrite = Box::new(write);
        (AsyncDecoder::new(read), AsyncEncoder::new(write))
    }
}
}

/// Synchronous helpers
pub mod sync {
    use crate::{Decoder, Encoder};
    use std::sync::{Arc, Mutex};

    /// A boxed [`std::io::Read`] trait object
    pub type BoxedRead = Box<dyn std::io::Read + Send>;
    /// A boxed [`std::io::Write`] trait object
    pub type BoxedWrite = Box<dyn std::io::Write + Send>;

    /// A boxed [`Decoder`]
    pub type BoxedDecoder = Decoder<BoxedRead>;

    /// A boxed [`Encoder`]
    pub type BoxedEncoder = Encoder<BoxedWrite>;

    /// Split an sync IO object and return [`Decoder`]/[`Encoder`]
    pub fn make_pair<IO>(io: IO) -> (Decoder<ReadHalf<IO>>, Encoder<WriteHalf<IO>>)
    where
        IO: std::io::Read + std::io::Write,
    {
        let (read, write) = split(io);
        (Decoder::new(read), Encoder::new(write))
    }

    /// Split an sync IO object and return type-erased [`Decoder`]/[`Encoder`]
    pub fn make_boxed_pair<IO>(io: IO) -> (BoxedDecoder, BoxedEncoder)
    where
        IO: std::io::Read + std::io::Write,
        IO: Send + 'static,
    {
        let (read, write) = split(io);
        let read: BoxedRead = Box::new(read);
        let write: BoxedWrite = Box::new(write);
        (Decoder::new(read), Encoder::new(write))
    }

    /// Splits this IO object into `Read` and `Write` halves
    fn split<IO>(io: IO) -> (ReadHalf<IO>, WriteHalf<IO>)
    where
        IO: std::io::Read + std::io::Write,
    {
        let this = Arc::new(Mutex::new(io));
        (ReadHalf(this.clone()), WriteHalf(this))
    }

    #[derive(Clone)]
    /// Read half of an IO object
    pub struct ReadHalf<T>(Arc<Mutex<T>>);

    impl<T> std::fmt::Debug for ReadHalf<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ReadHalf").finish()
        }
    }

    impl<T> std::io::Read for ReadHalf<T>
    where
        T: std::io::Read,
    {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().read(buf)
        }
    }

    #[derive(Clone)]
    /// Write half of an IO object
    pub struct WriteHalf<T>(Arc<Mutex<T>>);

    impl<T> std::fmt::Debug for WriteHalf<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("WriteHalf").finish()
        }
    }

    impl<T> std::io::Write for WriteHalf<T>
    where
        T: std::io::Write,
    {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }
}
