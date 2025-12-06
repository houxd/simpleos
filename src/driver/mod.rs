use crate::Result;

pub trait Driver {
    fn driver_init(&mut self) -> Result<()>;
    fn driver_deinit(&mut self) -> Result<()>;
}

pub mod device;
pub mod mtd;
pub mod fs;
pub mod gpio;
pub mod i2c;
pub mod rtc;
pub mod spi;
pub mod systick;
pub mod uart;
pub mod lazy_init;

pub use crate::device_table;
pub use crate::device_struct;
pub use crate::lazy_init;