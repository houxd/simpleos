use crate::driver::Driver;
use anyhow::Result;

pub trait I2cDriver: Driver {
    fn i2c_write(&mut self, addr: u16, data: &[u8]) -> Result<()>;
    fn i2c_read(&mut self, addr: u16, buffer: &mut [u8]) -> Result<()>;
}
