use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct SleepMsFuture {
    escape: u32,
}

impl Future for SleepMsFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_tick = crate::os_interface().get_system_ms();
        if current_tick >= self.escape {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[allow(unused)]
pub fn sleep_ms(ticks: u32) -> SleepMsFuture {
    unsafe {
        SleepMsFuture {
            escape: ticks + crate::os_interface().get_system_ms(),
        }
    }
}

#[allow(unused)]
pub fn get_system_ms() -> u32 {
    crate::os_interface().get_system_ms()
}