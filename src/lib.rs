#![no_std]

pub extern crate alloc;
pub use core;

pub mod sys;
pub mod driver;
// pub mod bindings;

#[cfg(feature = "util")]
pub mod util;

pub trait OsInterface {
    fn get_tick_count(&self) -> u32;
}

static mut OS_INTERFACE: Option<&'static dyn OsInterface> = None;

const fn os_interface() -> &'static dyn OsInterface {
    unsafe { OS_INTERFACE.expect("OS interface not initialized! Call simpleos_init() first") }
}

pub fn simpleos_init(interface: &'static impl OsInterface) {
    unsafe {
        OS_INTERFACE = Some(interface);
    }
}
