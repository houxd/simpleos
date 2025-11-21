pub mod i2c;
pub mod rtc;
pub mod uart;
// pub mod fs;

use crate::sys::PinBoxFuture;
use anyhow::Result;

pub trait Driver {
    fn driver_init(&mut self) -> PinBoxFuture<'_, Result<()>>;
    fn driver_deinit(&mut self) -> PinBoxFuture<'_, Result<()>>;
}
