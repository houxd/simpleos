use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use alloc::boxed::Box;

pub struct Select2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    future1: Pin<Box<F1>>,
    future2: Pin<Box<F2>>,
    future1_done: bool,
    future2_done: bool,
}

impl<F1, F2> Select2<F1, F2>
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
        }
    }
}

pub enum Select2Output<T1, T2> {
    Future1(T1),
    Future2(T2),
}

impl<F1, F2> Future for Select2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = Select2Output<F1::Output, F2::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        // 先轮询第一个 future
        if !this.future1_done {
            match this.future1.as_mut().poll(cx) {
                Poll::Ready(output) => {
                    this.future1_done = true;
                    return Poll::Ready(Select2Output::Future1(output));
                }
                Poll::Pending => {}
            }
        }

        // 再轮询第二个 future
        if !this.future2_done {
            match this.future2.as_mut().poll(cx) {
                Poll::Ready(output) => {
                    this.future2_done = true;
                    return Poll::Ready(Select2Output::Future2(output));
                }
                Poll::Pending => {}
            }
        }

        // 如果都未完成,继续等待
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

impl<F1, F2> Unpin for Select2<F1, F2>
where
    F1: Future,
    F2: Future,
{
}

#[allow(unused)]
pub fn select<F1, F2>(future1: F1, future2: F2) -> Select2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    Select2::new(future1, future2)
}

#[macro_export]
macro_rules! select {
    // 简化版本: 只支持两个分支
    {
        $pat1:pat = $fut1:expr => $body1:block,
        $pat2:pat = $fut2:expr => $body2:block $(,)?
    } => {{
        match $crate::sys::select($fut1, $fut2).await {
            $crate::sys::Select2Output::Future1($pat1) => $body1,
            $crate::sys::Select2Output::Future2($pat2) => $body2,
        }
    }};
}