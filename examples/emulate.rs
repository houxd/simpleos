use simpleos::alloc::boxed::Box;
use simpleos::device_table;
use simpleos::driver::systick::SysTickDriver;
use simpleos::driver::Driver;
use simpleos::SimpleOs;
use simpleos::singleton;
use simpleos::sys::sleep_ms;
use simpleos::sys::Executor;
use simpleos::util;
use simpleos::Result;

async fn task1() {
    loop {
        println!("task1");
        sleep_ms(1000).await;
    }
}

async fn sub_test() {
    sleep_ms(1000).await;
}

async fn task2() {
    loop {
        println!("task2");
        let data = b"Hello, world!";
        let crc = util::crc16(data);
        println!("CRC16 of {:?} is {:04X}", data, crc);
        sub_test().await;
    }
}

struct SysTickEmulate;
impl Driver for SysTickEmulate {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}
impl SysTickDriver for SysTickEmulate {
    fn get_system_ms(&self) -> u32 {
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

struct BoardEmulate;
device_table!(BoardEmulate, {
    SysTick0: SysTickEmulate = SysTickEmulate{},
});
impl simpleos::driver::device::Device for BoardEmulate {
    fn default_console(&self) -> &'static mut dyn simpleos::console::ConsoleDriver {
        unimplemented!()
    }
    fn default_systick(&self) -> &'static mut dyn simpleos::driver::systick::SysTickDriver {
        SysTick0::dev()
    }
    
    fn init(&self) {
        
    }
}

fn main() {
    SimpleOs::init(&BoardEmulate);
    Executor::spawn("task1", Box::pin(task1()));
    Executor::spawn("task2", Box::pin(task2()));
    Executor::run();
}
