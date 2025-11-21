use crate::println;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Error: {}", info);
    loop {}
}
