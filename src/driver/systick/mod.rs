use crate::driver::Driver;

pub trait SysTickDriver: Driver {
    fn get_system_ms(&self) -> u32;
    fn delay_ms(&self, ms: u32) {
        let start = self.get_system_ms();
        while self.get_system_ms().wrapping_sub(start) < ms {}
    }
}
