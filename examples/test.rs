use std::sync::Mutex;

static VAL: Mutex<u32> = Mutex::new(42);
fn main() {
    *VAL.lock().unwrap() = 10;
    println!("VAL = {}", *VAL.lock().unwrap());
}
