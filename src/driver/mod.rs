use anyhow::Result;

pub trait Driver {
    fn driver_init(&mut self) -> Result<()>;
    fn driver_deinit(&mut self) -> Result<()>;
}

pub mod device;
pub mod systick;
// pub mod fs;
pub mod gpio;
pub mod i2c;
pub mod rtc;
pub mod spi;
pub mod uart;

pub use crate::device_table;
