// use crate::driver::stm32::Stm32;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

unsafe extern "C" {
    fn get_tick_count() -> u32;
}

pub struct SleepFuture {
    escape: u32,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_tick = unsafe { get_tick_count() };
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
            escape: ticks + get_tick_count(),
        }
    }
}
