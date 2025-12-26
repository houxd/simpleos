use crate::{
    driver::cpu::CpuDriver, driver::systick::SysTickDriver, driver::tty::TtyDriver, singleton,
};

pub trait Device {
    fn get_cpu(&self) -> &'static mut dyn CpuDriver;
    fn get_tty(&self) -> &'static mut dyn TtyDriver;
    fn get_systick(&self) -> &'static mut dyn SysTickDriver;
}

pub struct SimpleOs {
    device: Option<&'static dyn Device>,
}
singleton!(SimpleOs { device: None });

impl SimpleOs {
    pub fn init(device: &'static dyn Device) {
        SimpleOs::get_mut().device = Some(device);
    }
    pub fn is_initialized() -> bool {
        SimpleOs::get_mut().device.is_some()
    }
    fn device() -> &'static dyn Device {
        if let Some(device) = SimpleOs::get_mut().device {
            device
        } else {
            panic!("SimpleOs is not initialized!");
        }
    }
    pub fn cpu() -> &'static mut dyn CpuDriver {
        SimpleOs::device().get_cpu()
    }
    pub fn tty() -> &'static mut dyn TtyDriver {
        SimpleOs::device().get_tty()
    }
    pub fn systick() -> &'static mut dyn SysTickDriver {
        SimpleOs::device().get_systick()
    }
}

mod join;
mod print;
mod select;
mod sleep;
mod yield_now;

pub use join::*;
pub use select::*;
pub use sleep::*;
pub use yield_now::*;

pub use crate::print;
pub use crate::println;