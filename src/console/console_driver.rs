
pub trait ConsoleDriver {
    fn console_getc(&mut self) -> Option<u8>;
    fn console_putc(&mut self, byte: u8) -> bool;
    fn console_flush(&mut self);

    fn console_flush_rx(&mut self) {
        while self.console_getc().is_some() {}
    }
    #[allow(unused)]
    fn console_read(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for byte in buffer.iter_mut() {
            if let Some(b) = self.console_getc() {
                *byte = b;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
    #[allow(unused)]
    fn console_write(&mut self, data: &[u8]) {
        for byte in data.iter() {
            if *byte == b'\n' {
                self.console_putc(b'\r');
            }
            self.console_putc(*byte);
        }
    }
}