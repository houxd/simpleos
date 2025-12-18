use crate::sys::SimpleOs;
use alloc::format;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    SimpleOs::cpu().cpu_panic(format!("Error: {}", info));
}
