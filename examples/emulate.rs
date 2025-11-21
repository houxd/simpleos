use simpleos::OsInterface;
use simpleos::alloc::boxed::Box;
use simpleos::simpleos_init;
use simpleos::sys::Executor;
use simpleos::sys::sleep;
use simpleos::sys::yield_now;
use simpleos::util;

async fn task1() {
    loop {
        println!("task1");
        yield_now().await;
        sleep(1000).await; // 模拟延时
    }
}

async fn sub_test() {
    sleep(1000).await; // 模拟延时
    println!("sub_test");
}

async fn task2() {
    loop {
        let data = b"Hello, world!";
        let crc = util::crc16(data);
        println!("CRC16 of {:?} is {:04X}", data, crc);

        sub_test().await; // 调用另一个异步函数
        println!("task2");
        sleep(2000).await; // 模拟延时
    }
}

struct OsInterfaceEmulate;
impl OsInterface for OsInterfaceEmulate {
    fn get_tick_count(&self) -> u32 {
        static mut BOOT_TIMESTAMP: u128 = 0;

        fn get_ms_from_unix_epoch() -> u128 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        }

        unsafe {
            let current_time = get_ms_from_unix_epoch();
            (current_time - BOOT_TIMESTAMP) as u32
        }
    }
}

fn main() {
    simpleos_init(&OsInterfaceEmulate);
    Executor::spawn("task1", Box::pin(task1()));
    Executor::spawn("task2", Box::pin(task2()));
    Executor::run();
}
