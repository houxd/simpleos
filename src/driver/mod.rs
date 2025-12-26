use crate::Result;

pub trait Driver {
    fn driver_init(&mut self) -> Result<()>;
    fn driver_deinit(&mut self) -> Result<()>;
    fn driver_dev_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }
}

pub mod fs;
pub mod gpio;
pub mod i2c;
pub mod lazy_init;
pub mod mtd;
pub mod rtc;
pub mod spi;
pub mod systick;
pub mod uart;
pub mod cpu;
pub mod tty;

pub use crate::lazy_init;
