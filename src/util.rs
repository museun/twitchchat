#![allow(dead_code)] // some of these won't be used for now
use futures_lite::future::{ready, Ready};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub fn name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

/// Note: This does not work (the way you'd expect) with compound types.
///
/// This is mainly used to turn 'twitchchat::messages::GlobalUserState' et.al
/// into 'GlobalUserState'
pub fn trim_type_name<T>() -> &'static str {
    let ty = std::any::type_name::<T>();
    if ty.contains('<') {
        return ty;
    }
    ty.rsplit("::").next().unwrap_or(ty)
}

pub fn trim_type_name_val<T>(_: &T) -> &'static str {
    let ty = std::any::type_name::<T>();
    if ty.contains('<') {
        return ty;
    }
    ty.rsplit("::").next().unwrap_or(ty)
}

pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// TODO if we make this clonable it can be used in the Writers
pub struct Notify {
    rx: crate::channel::Receiver<()>,
    triggered: bool,
}

impl std::fmt::Debug for Notify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Notify").finish()
    }
}

impl Notify {
    pub fn new() -> (Self, NotifyHandle) {
        let (tx, rx) = crate::channel::bounded(1);
        let this = Self {
            rx,
            triggered: false,
        };
        (this, NotifyHandle { tx })
    }

    pub async fn wait(&mut self) {
        // cache the wait
        if self.triggered {
            return;
        }

        use futures_lite::StreamExt as _;
        let _ = self.rx.next().await;
        self.triggered = true;
    }
}

/// A notify handle for sending a single-shot signal to the 'other side'
#[derive(Clone)]
pub struct NotifyHandle {
    tx: crate::channel::Sender<()>,
}

impl std::fmt::Debug for NotifyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotifyHandle").finish()
    }
}

impl NotifyHandle {
    /// Consumes the handle, notifying the other side.
    ///
    /// Returns false if the other side wasn't around any more
    pub async fn notify(self) -> bool {
        self.tx.send(()).await.is_ok()
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub use Either::{Left, Right};

pub trait FutExt
where
    Self: Future + Send + Sync + Sized,
    Self::Output: Send + Sync,
{
    fn either<F>(self, other: F) -> Or<Self, F>
    where
        F: Future + Send + Sync,
        F::Output: Send + Sync;

    fn first<F>(self, other: F) -> Or<Self, F>
    where
        F: Future + Send + Sync,
        F::Output: Send + Sync;

    fn now_or_never(self) -> Or<Self, Ready<()>>;
}

impl<T> FutExt for T
where
    T: Future + Send + Sync,
    T::Output: Send + Sync,
{
    fn either<F>(self, right: F) -> Or<Self, F>
    where
        F: Future + Send + Sync,
        F::Output: Send + Sync,
    {
        let left = self;
        Or {
            left,
            right,
            biased: false,
        }
    }

    fn first<F>(self, right: F) -> Or<Self, F>
    where
        F: Future + Send + Sync,
        F::Output: Send + Sync,
    {
        let left = self;
        Or {
            left,
            right,
            biased: true,
        }
    }

    fn now_or_never(self) -> Or<Self, Ready<()>> {
        self.first(ready(()))
    }
}

pin_project_lite::pin_project! {
    pub struct Or<A,B> {
        #[pin]
        left: A,

        #[pin]
        right: B,

        biased: bool,
    }
}

impl<A, B> Future for Or<A, B>
where
    A: Future + Send + Sync,
    A::Output: Send + Sync,

    B: Future + Send + Sync,
    A::Output: Send + Sync,
{
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        macro_rules! poll {
            ($expr:ident => $map:expr) => {
                if let Poll::Ready(t) = this.$expr.poll(cx).map($map) {
                    return Poll::Ready(t);
                }
            };
        }

        if *this.biased {
            poll!(left => Left);
            poll!(right => Right);
            return Poll::Pending;
        }

        if fastrand::bool() {
            poll!(left => Left);
            poll!(right => Right);
        } else {
            poll!(right => Right);
            poll!(left => Left);
        }

        Poll::Pending
    }
}
