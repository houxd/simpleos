use crate::driver::Driver;

pub trait TtyDriver: Driver {
    fn tty_getc(&mut self) -> Option<u8>;
    fn tty_putc(&mut self, byte: u8);
    fn tty_flush(&mut self);
    fn tty_get_break(&mut self) -> bool;

    #[allow(unused)]
    fn tty_clear_rx(&mut self) {
        while self.tty_getc().is_some() {}
        while self.tty_get_break() {}
    }
    #[allow(unused)]
    fn tty_read(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for byte in buffer.iter_mut() {
            if let Some(b) = self.tty_getc() {
                *byte = b;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
    #[allow(unused)]
    fn tty_write(&mut self, data: &[u8]) {
        for byte in data.iter() {
            if *byte == b'\n' {
                self.tty_putc(b'\r');
            }
            self.tty_putc(*byte);
        }
    }
}