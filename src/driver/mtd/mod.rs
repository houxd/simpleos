use crate::driver::Driver;
use anyhow::Result;

pub trait MtdDriver: Driver {
    fn mtd_read(&mut self, addr: u32, buffer: &mut [u8]) -> Result<()>;
    fn mtd_write(&mut self, addr: u32, data: &[u8]) -> Result<()>;
    fn mtd_erase(&mut self, addr: u32, size: u32) -> Result<()>;
    fn size(&mut self) -> u32;
    fn erase_size(&mut self) -> u32;
    fn write_size(&mut self) -> u32;
}

pub mod sfud;