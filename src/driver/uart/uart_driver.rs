pub trait UartDriver {
    fn drv_init(&mut self);
    fn drv_write(&mut self, data: &[u8]);
}
