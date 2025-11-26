use crate::driver::Driver;

pub trait GpioDriver: Driver {
    fn gpio_write(&mut self, value: bool);
    fn gpio_read(&mut self) -> bool;
    fn gpio_toggle(&mut self) {
        let current = self.gpio_read();
        self.gpio_write(!current);
    }
    fn gpio_set_high(&mut self) {
        self.gpio_write(true);
    }
    fn gpio_set_low(&mut self) {
        self.gpio_write(false);
    }
}


pub mod dummy_gpio;