use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::SimpleOs;

pub struct SleepMsFuture {
    escape: u32,
}

impl Future for SleepMsFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_tick = SimpleOs::device().default_systick().get_system_ms();
        if current_tick >= self.escape {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[allow(unused)]
pub fn sleep_ms(ms: u32) -> SleepMsFuture {
    unsafe {
        SleepMsFuture {
            escape: ms + SimpleOs::device().default_systick().get_system_ms(),
        }
    }
}

#[allow(unused)]
#[inline]
pub fn get_system_ms() -> u32 {
    SimpleOs::device().default_systick().get_system_ms()
}

#[allow(unused)]
#[inline]
pub fn delay_ms(ms: u32) {
    SimpleOs::device().default_systick().delay_ms(ms);
}
