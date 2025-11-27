use crate::driver::Driver;
use anyhow::Result;

pub trait MtdDriver: Driver {
    fn mtd_read(&mut self, addr: u32, buffer: &mut [u8]) -> Result<()>;
    fn mtd_write(&mut self, addr: u32, data: &[u8]) -> Result<()>;
    fn mtd_erase(&mut self, addr: u32, size: u32) -> Result<()>;
    // fn block_size(&self) -> u32;
    // fn total_size(&self) -> u32;
}

pub mod sfud;