use crate::driver::{Driver, gpio::GpioDriver};
use alloc::vec;
use anyhow::Result;

pub trait SpiDriver: Driver {
    fn spi_write_read(&mut self, data: &[u8], buffer: &mut [u8]) -> Result<()>;
    fn spi_cs_pin(&mut self) -> &mut dyn GpioDriver;
    fn spi_cs_activate(&mut self) {
        self.spi_cs_pin().gpio_set_low();
    }
    fn spi_cs_deactivate(&mut self) {
        self.spi_cs_pin().gpio_set_high();
    }
    fn spi_write(&mut self, data: &[u8]) -> Result<()> {
        self.spi_write_read(data, &mut [])
    }
    fn spi_read(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.spi_write_read(&vec![0; buffer.len()], buffer)
    }
}
