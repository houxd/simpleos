use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::boxed::Box;

#[allow(unused)]
pub struct Join2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    future1: Pin<Box<F1>>,
    future2: Pin<Box<F2>>,
    future1_done: bool,
    future2_done: bool,
    output1: Option<F1::Output>,
    output2: Option<F2::Output>,
}

impl<F1, F2> Join2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    pub fn new(future1: F1, future2: F2) -> Self {
        Self {
            future1: Box::pin(future1),
            future2: Box::pin(future2),
            future1_done: false,
            future2_done: false,
            output1: None,
            output2: None,
        }
    }
}

impl<F1, F2> Future for Join2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if !this.future1_done {
            match this.future1.as_mut().poll(cx) {
                Poll::Ready(output) => {
                    this.output1 = Some(output);
                    this.future1_done = true;
                }
                Poll::Pending => {}
            }
        }

        if !this.future2_done {
            match this.future2.as_mut().poll(cx) {
                Poll::Ready(output) => {
                    this.output2 = Some(output);
                    this.future2_done = true;
                }
                Poll::Pending => {}
            }
        }

        if this.future1_done && this.future2_done {
            let output1 = this.output1.take().unwrap();
            let output2 = this.output2.take().unwrap();
            Poll::Ready((output1, output2))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl<F1, F2> Unpin for Join2<F1, F2>
where
    F1: Future,
    F2: Future,
{
}

#[allow(unused)]
pub fn join<F1, F2>(future1: F1, future2: F2) -> Join2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    Join2::new(future1, future2)
}

// 改进版宏,自动包含 .await
#[macro_export]
macro_rules! join {
    ($fut1:expr, $fut2:expr) => {
        async {
            $crate::sys::join($fut1, $fut2).await
        }.await
    };
    ($fut1:expr, $fut2:expr, $fut3:expr) => {
        async {
            let ((r1, r2), r3) = $crate::sys::join(
                $crate::sys::join($fut1, $fut2),
                $fut3
            ).await;
            (r1, r2, r3)
        }.await
    };
    ($fut1:expr, $fut2:expr, $fut3:expr, $fut4:expr) => {
        async {
            let ((r1, r2), (r3, r4)) = $crate::sys::join(
                $crate::sys::join($fut1, $fut2),
                $crate::sys::join($fut3, $fut4)
            ).await;
            (r1, r2, r3, r4)
        }.await
    };
}