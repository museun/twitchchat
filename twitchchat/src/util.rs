use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R>
where
    L: Future,
    R: Future,
    Self: Future<Output = Either<L::Output, R::Output>>,
{
    pub fn pair(left: L, right: R) -> (Self, Self) {
        (Either::Left(left), Either::Right(right))
    }

    pub async fn select(left: L, right: R) -> Either<L::Output, R::Output> {
        let (left, right) = Self::pair(left, right);
        futures_lite::future::race(left, right).await
    }
}

impl<L: Future + Unpin, R: Future + Unpin> Future for Either<L, R> {
    type Output = Either<L::Output, R::Output>;
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        use futures_lite::{pin, ready};

        match &mut *self.as_mut() {
            Either::Left(left) => {
                pin!(left);
                let left = ready!(left.poll(ctx));
                Poll::Ready(Either::Left(left))
            }
            Either::Right(right) => {
                pin!(right);
                let right = ready!(right.poll(ctx));
                Poll::Ready(Either::Right(right))
            }
        }
    }
}
