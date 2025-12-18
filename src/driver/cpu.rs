use alloc::string::String;

use crate::println;

pub trait CpuDriver {
    fn cpu_reset(&mut self) -> !;
    fn cpu_panic(&mut self, panic_info: String) -> ! {
        println!("Panic: {}", panic_info);
        loop {}
    }
}
