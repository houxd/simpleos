use simpleos::alloc::boxed::Box;
use simpleos::console::ConsoleDriver;
use simpleos::driver::cpu::CpuDriver;
use simpleos::driver::lazy_init::LazyInit;
use simpleos::driver::systick::SysTickDriver;
use simpleos::driver::Driver;
use simpleos::executor::Executor;
use simpleos::singleton;
use simpleos::sys::sleep_ms;
use simpleos::sys::Device;
use simpleos::sys::SimpleOs;
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

struct CpuEmulate;
impl Driver for CpuEmulate {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}
impl CpuDriver for CpuEmulate {
    fn cpu_reset(&mut self) -> ! {
        panic!("System reset called in emulation.");
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

struct ConsoleDriverEmulate;
impl Driver for ConsoleDriverEmulate {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }
    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}
impl ConsoleDriver for ConsoleDriverEmulate {
    fn console_getc(&mut self) -> Option<u8> {
        None
    }

    fn console_putc(&mut self, byte: u8) -> bool {
        print!("{}", byte as char);
        true
    }

    fn console_flush(&mut self) {}
}

struct BoardEmulate {
    cpu0: LazyInit<CpuEmulate>,
    systick0: LazyInit<SysTickEmulate>,
    console0: LazyInit<ConsoleDriverEmulate>,
}

singleton!(BoardEmulate {
    cpu0: LazyInit::new(|| CpuEmulate {}),
    systick0: LazyInit::new(|| SysTickEmulate {}),
    console0: LazyInit::new(|| ConsoleDriverEmulate {}),
});

impl Device for BoardEmulate {
    fn get_cpu(&self) -> &'static mut dyn simpleos::driver::cpu::CpuDriver {
        BoardEmulate::get_mut().cpu0.get_or_init()
    }
    
    fn get_console(&self) -> &'static mut dyn ConsoleDriver {
        BoardEmulate::get_mut().console0.get_or_init()
    }
    
    fn get_systick(&self) -> &'static mut dyn SysTickDriver {
        BoardEmulate::get_mut().systick0.get_or_init()
    }
}

fn main() {
    SimpleOs::init(BoardEmulate::get_mut());
    Executor::spawn("task1", Box::pin(task1()));
    Executor::spawn("task2", Box::pin(task2()));
    Executor::run();
}
