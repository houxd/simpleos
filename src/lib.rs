#![no_std]

pub extern crate alloc;
use crate::driver::device::Device;
pub use anyhow::anyhow;
pub use anyhow::Result;
pub use core;

pub mod bindings;
pub mod console;
pub mod driver;
pub mod sys;

#[cfg(feature = "util")]
pub mod util;

pub struct SimpleOs {
    device: Option<&'static dyn Device>,
}
singleton!(SimpleOs { device: None });

impl SimpleOs {
    pub fn init(device: &'static dyn Device) {
        SimpleOs::ref_mut().device = Some(device);
    }
    pub fn is_initialized() -> bool {
        SimpleOs::ref_mut().device.is_some()
    }
    pub fn device() -> &'static dyn Device {
        SimpleOs::ref_mut().device.as_deref().unwrap()
    }
}
