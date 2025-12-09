use crate::driver::Driver;

pub trait SysTickDriver: Driver {
    fn get_system_ms(&self) -> u32;
}
