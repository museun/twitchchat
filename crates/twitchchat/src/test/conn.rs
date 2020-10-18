use std::{
    future::Future,
    io::{Error, ErrorKind, Result},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_mutex::Mutex;
use futures_lite::io::*;

/// A test connection that you can use to insert into and read messages from.
#[derive(Default, Debug, Clone)]
pub struct TestConn {
    read: Arc<Mutex<Cursor<Vec<u8>>>>,
    write: Arc<Mutex<Cursor<Vec<u8>>>>,
}

fn take_cursor<T: Default>(cursor: &mut Cursor<T>) -> T {
    let out = std::mem::take(cursor.get_mut());
    cursor.set_position(0);
    out
}

impl TestConn {
    /// Create a new TestConn
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the instance and returning a clone
    pub fn reset(&self) -> Self {
        futures_lite::future::block_on(async {
            take_cursor(&mut *self.read.lock().await);
            take_cursor(&mut *self.write.lock().await);
        });

        self.clone()
    }

    /// Write `data` to the underlying buffers.
    ///
    /// Whatever uses `AsyncRead` on this type will read from this buffer
    pub async fn write_data(&self, data: impl AsRef<[u8]>) {
        let mut read = self.read.lock().await;
        let p = read.position();
        read.write_all(data.as_ref()).await.unwrap();
        read.set_position(p);
    }

    /// Read all of the lines written via `AsyncWrite`
    pub async fn read_all_lines(&self) -> Result<Vec<String>> {
        let data = take_cursor(&mut *self.write.lock().await);
        Ok(String::from_utf8(data)
            .map_err(|err| Error::new(ErrorKind::Other, err))?
            .lines()
            .map(|s| format!("{}\r\n", s))
            .collect())
    }

    /// Read the first line written via an `AsyncWrite`
    pub async fn read_line(&self) -> Result<String> {
        let mut write = self.write.lock().await;

        write.set_position(0);
        let mut line = Vec::new();
        let mut buf = [0_u8; 1]; // speed doesn't matter.

        while !line.ends_with(b"\r\n") {
            write.read_exact(&mut buf).await?;
            line.extend_from_slice(&buf);
        }

        String::from_utf8(line).map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

macro_rules! impls {
    ($($ty:ty)*) => {
        $(
        impl AsyncRead for $ty {
            fn poll_read(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &mut [u8],
            ) -> Poll<Result<usize>> {
                let this = self.get_mut();

                let fut = this.read.lock();
                futures_lite::pin!(fut);

                let mut guard = futures_lite::ready!(fut.poll(cx));
                let guard = &mut *guard;
                futures_lite::pin!(guard);
                guard.poll_read(cx, buf)
            }
        }

        impl AsyncWrite for $ty {
            fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
                let this = self.get_mut();

                let fut = this.write.lock();
                futures_lite::pin!(fut);

                let mut guard = futures_lite::ready!(fut.poll(cx));
                guard.get_mut().extend_from_slice(buf);

                let fut = guard.seek(std::io::SeekFrom::Current(buf.len() as _));
                futures_lite::pin!(fut);
                if let Err(err) = futures_lite::ready!(fut.poll(cx)) {
                    return Poll::Ready(Err(err))
                }

                Poll::Ready(Ok(buf.len()))
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
                Poll::Ready(Ok(()))
            }
        }
        )*
    };
}

impls! {
    &TestConn
    TestConn
}

/// TODO: Something
pub fn create_mock_connection() -> TestConn {
    todo!()
}
