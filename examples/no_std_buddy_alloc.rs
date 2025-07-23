#![no_std]
#![no_main]

// 使用BuddlyAlloc 从静态数组分配Heap
use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};
use core::ptr::addr_of_mut;

const FAST_HEAP_SIZE: usize = 32 * 1024; // 32 KB
const HEAP_SIZE: usize = 1024 * 1024; // 1M
const LEAF_SIZE: usize = 16;

#[repr(align(64))]
struct Heap<const S: usize>([u8; S]);

static mut FAST_HEAP: Heap<FAST_HEAP_SIZE> = Heap([0u8; FAST_HEAP_SIZE]);
static mut HEAP: Heap<HEAP_SIZE> = Heap([0u8; HEAP_SIZE]);

#[cfg_attr(not(test), global_allocator)]
#[allow(unused)]
static ALLOC: NonThreadsafeAlloc = {
    let fast_param = FastAllocParam::new(addr_of_mut!(FAST_HEAP).cast(), FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(addr_of_mut!(HEAP).cast(), HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};

// 提供panic含函数
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// 提供tick函数
static mut TICK_COUNT: u32 = 0;
#[unsafe(no_mangle)]
pub extern "C" fn get_tick_count() -> u32 {
    unsafe {
        TICK_COUNT += 10000; // 模拟每次调用增加 tick
        TICK_COUNT
    }
}

// 测试异步函数
extern crate alloc;
use alloc::boxed::Box;
use masy::delay::delay;
use masy::executor::Executor;

async fn task1() {
    loop {
        //println!("task1");
        delay(1000).await; // 模拟延时
    }
}

async fn sub_task() {
    delay(1000).await; // 模拟延时
    // println!("sub_task");
}

async fn task2() {
    loop {
        sub_task().await; // 调用另一个异步函数
        //println!("task2");
        delay(2000).await; // 模拟延时
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let mut executor = Executor::new();
    executor.spawn(Box::pin(task1()));
    executor.spawn(Box::pin(task2()));
    executor.run();
}

