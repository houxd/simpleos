use anyhow::Result;

pub trait Driver {
    fn driver_init(&mut self) -> Result<()>;
    fn driver_deinit(&mut self) -> Result<()>;
}

pub mod i2c;
pub mod rtc;
pub mod uart;
// pub mod fs;
pub mod device;

pub use crate::device;
