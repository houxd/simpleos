//提供tick
use std::time::{SystemTime, UNIX_EPOCH};

static mut BOOT_TIMESTAMP: u64 = 0;

fn get_ms_from_unix_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn get_tick_count() -> u32 {
    unsafe {
        let current_time = get_ms_from_unix_epoch();
        (current_time - BOOT_TIMESTAMP) as u32
    }
}

// 测试异步函数
extern crate alloc;
use alloc::boxed::Box;
use simpleos::delay;
use simpleos::Executor;
use simpleos::yield_now;

async fn task1() {
    loop {
        println!("task1");
        yield_now().await;
        delay(1000).await; // 模拟延时
    }
}

async fn sub_test() {
    delay(1000).await; // 模拟延时
    println!("sub_test");
}

async fn task2() {
    loop {
        sub_test().await; // 调用另一个异步函数
        println!("task2");
        delay(2000).await; // 模拟延时
    }
}

fn main() {
    let mut executor = Executor::new();
    executor.spawn(Box::pin(task1()));
    executor.spawn(Box::pin(task2()));
    executor.run();
}
