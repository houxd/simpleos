use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

unsafe extern "C" {
    fn get_tick_count() -> u32;
}

pub struct Delay {
    escape: u32,
}

impl Future for Delay {
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
pub fn delay(ticks: u32) -> Delay {
    unsafe {
        Delay {
            escape: ticks + get_tick_count(),
        }
    }
}
