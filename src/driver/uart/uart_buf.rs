use crate::util::RingBuf;
use crate::{driver::uart::uart_driver::UartDriver, sys::ConsoleIO};
use core::{any::Any, fmt};

/// UART控制器, 带收发缓冲区
/// 注意需要全局不能move clone, 因为HAL库会记住rx_byte的地址
/// 全局创建后请调用 init() 方法初始化接收
pub struct UartBuf<const RX_SIZE: usize = 128, const TX_SIZE: usize = RX_SIZE> {
    driver: &'static mut dyn UartDriver,
    pub rx: RingBuf<u8, RX_SIZE>,
    pub tx: RingBuf<u8, TX_SIZE>,
}

impl<const RX_SIZE: usize, const TX_SIZE: usize> fmt::Debug for UartBuf<RX_SIZE, TX_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Uart")
            .field("rx_size", &RX_SIZE)
            .field("tx_size", &TX_SIZE)
            .finish()
    }
}

#[allow(unused)]
impl<const RX_SIZE: usize, const TX_SIZE: usize> UartBuf<RX_SIZE, TX_SIZE> {
    pub fn new(uart_driver: &'static mut dyn UartDriver) -> Self {
        uart_driver.drv_init();
        UartBuf {
            driver: uart_driver,
            rx: RingBuf::new(),
            tx: RingBuf::new(),
        }
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for byte in buffer.iter_mut() {
            if let Some(b) = self.read_byte() {
                *byte = b;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        self.rx.pop()
    }

    pub fn clear_rx(&mut self) {
        while self.read_byte().is_some() {}
    }

    pub fn write_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.write_byte(b);
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        for &byte in data {
            self.write_byte(byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> bool {
        if self.tx.is_full() {
            self.flush();
        }
        self.tx.push(byte)
    }

    pub fn flush(&mut self) {
        if self.tx.is_empty() {
            return;
        }

        let mut buffer = [0u8; TX_SIZE];
        let mut count = 0;
        while let Some(byte) = self.tx.pop() {
            buffer[count] = byte;
            count += 1;
            if count >= TX_SIZE {
                break;
            }
        }

        self.driver.drv_write(&buffer[..count]);
    }

    /// 用于中断中调用来标记接收到字节, 并且将下一个接收字节返回
    pub fn rx_complete(&mut self, byte: u8) {
        self.rx.push(byte);
    }
}

impl ConsoleIO for UartBuf {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get(&mut self) -> Option<u8> {
        self.read_byte()
    }

    fn put(&mut self, byte: u8) -> bool {
        self.write_byte(byte)
    }

    fn flush(&mut self) {
        self.flush();
    }
}
