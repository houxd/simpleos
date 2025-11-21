use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct SleepFuture {
    escape: u32,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_tick = crate::os_interface().get_tick_count();
        if current_tick >= self.escape {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[allow(unused)]
pub fn sleep(ticks: u32) -> SleepFuture {
    unsafe {
        SleepFuture {
            escape: ticks + crate::os_interface().get_tick_count(),
        }
    }
}
