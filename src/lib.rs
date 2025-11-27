#![no_std]

pub extern crate alloc;
use crate::driver::systick::SysTickDriver;
pub use anyhow::anyhow;
pub use anyhow::Result;
pub use core;

// pub mod bindings;
pub mod console;
pub mod driver;
pub mod sys;

#[cfg(feature = "util")]
pub mod util;

pub struct SimpleOs {
    systick: Option<&'static mut dyn SysTickDriver>,
}
singleton!(SimpleOs { systick: None });

impl SimpleOs {
    pub fn init(systick: &'static mut dyn SysTickDriver) {
        SimpleOs::ref_mut().systick = Some(systick);
    }
    pub fn device() -> &'static mut dyn SysTickDriver {
        SimpleOs::ref_mut().systick.as_deref_mut().unwrap()
    }
}
