use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub fn name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct Notify {
    rx: crate::channel::Receiver<()>,
}

impl Notify {
    pub fn new() -> (Self, NotifyHandle) {
        let (tx, rx) = crate::channel::bounded(1);
        (Self { rx }, NotifyHandle { tx })
    }

    pub async fn wait(&mut self) {
        use futures_lite::StreamExt as _;
        let _ = self.rx.next().await;
    }
}

#[derive(Clone)]
pub struct NotifyHandle {
    tx: crate::channel::Sender<()>,
}

impl NotifyHandle {
    pub async fn notify(self) -> bool {
        self.tx.send(()).await.is_ok()
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub trait FutExt
where
    Self: Future + Send + Sync + Sized,
    Self::Output: Send + Sync,
{
    fn either<F>(self, other: F) -> Or<Self, F>
    where
        F: Future + Send + Sync,
        F::Output: Send + Sync;
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
        Or { left, right }
    }
}

pin_project_lite::pin_project! {
    pub struct Or<A,B> {
        #[pin]
        left: A,

        #[pin]
        right: B,
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

        if fastrand::bool() {
            poll!(left => Either::Left);
            poll!(right => Either::Right);
        } else {
            poll!(right => Either::Right);
            poll!(left => Either::Left);
        }

        Poll::Pending
    }
}
