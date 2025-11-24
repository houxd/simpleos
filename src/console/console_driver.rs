
pub trait ConsoleDriver {
    fn csl_getc(&mut self) -> Option<u8>;
    fn csl_putc(&mut self, byte: u8) -> bool;
    fn csl_flush(&mut self);

    fn flush_rx(&mut self) {
        while self.csl_getc().is_some() {}
    }
    #[allow(unused)]
    fn read(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for byte in buffer.iter_mut() {
            if let Some(b) = self.csl_getc() {
                *byte = b;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
    #[allow(unused)]
    fn write(&mut self, data: &[u8]) {
        for byte in data.iter() {
            if *byte == b'\n' {
                self.csl_putc(b'\r');
            }
            self.csl_putc(*byte);
        }
    }
}