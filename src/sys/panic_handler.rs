use crate::println;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    // 基础的错误信息输出
    if let Some(location) = info.location() {
        println!(
            "Panic at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        println!("Panic occurred");
    }

    // 获取错误消息
    let msg = info.message();
    // 尝试获取静态字符串
    if let Some(s) = msg.as_str() {
        println!("Error message: {}", s);
    } else {
        println!("Error message: None");
    }

    loop {}
}