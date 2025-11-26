use crate::driver::gpio::GpioDriver;
use crate::driver::Driver;
use anyhow::Result;

pub struct DummyGpio {}

impl Driver for DummyGpio {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }
    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}

impl GpioDriver for DummyGpio {
    fn gpio_write(&mut self, _high: bool) {}
    fn gpio_read(&mut self) -> bool {
        false
    }
}
