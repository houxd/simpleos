use crate::{driver::tty::TtyDriver, driver::Driver, sys, util::RingBuf};
use anyhow::Result;

pub trait UartDriver<const RX_SIZE: usize = 512, const TX_SIZE: usize = RX_SIZE>:
    Driver + TtyDriver
{
    fn rx(&mut self) -> &mut RingBuf<u8, RX_SIZE>;
    fn tx(&mut self) -> &mut RingBuf<u8, TX_SIZE>;
    fn uart_write(&mut self, data: &[u8]);

    fn read_byte(&mut self) -> Option<u8> {
        self.rx().pop()
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.tx().is_full() {
            self.flush();
        }
        if self.tx().push(byte) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("UART TX FULL"))
        }
    }

    fn flush(&mut self) {
        if self.tx().is_empty() {
            return;
        }

        let mut buffer = [0u8; TX_SIZE];
        let mut count = 0;
        while let Some(byte) = self.tx().pop() {
            buffer[count] = byte;
            count += 1;
            if count >= TX_SIZE {
                break;
            }
        }

        self.uart_write(&buffer[..count]);
    }

    /// 用于中断中调用来标记接收到字节, 并且将下一个接收字节返回
    fn rx_complete(&mut self, byte: u8) {
        self.rx().push(byte);
    }

    fn read(&mut self, buffer: &mut [u8]) -> usize {
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

    fn getc(&mut self, timeout_ms: u32) -> impl crate::core::future::Future<Output = Option<u8>> {
        async move {
            let start_ms = sys::get_system_ms();
            loop {
                if let Some(b) = self.read_byte() {
                    return Some(b);
                }

                if sys::get_system_ms().wrapping_sub(start_ms) >= timeout_ms {
                    return None;
                }

                sys::sleep_ms(1).await;
            }
        }
    }

    // 等待一个数据帧, 当收到第一个字节后, 连续等等接收, 超过 byte_interval_ms 则认为帧结束
    fn read_frame(
        &mut self,
        buffer: &mut [u8],
        byte_interval_ms: u32,
    ) -> impl crate::core::future::Future<Output = usize> {
        async move {
            let mut count = 0;
            let mut last_byte_ms = sys::get_system_ms();

            loop {
                if let Some(b) = self.read_byte() {
                    if count < buffer.len() {
                        buffer[count] = b;
                        count += 1;
                    }
                    last_byte_ms = sys::get_system_ms();
                } else {
                    // 检查是否超时
                    if count > 0
                        && sys::get_system_ms().wrapping_sub(last_byte_ms) >= byte_interval_ms
                    {
                        break;
                    }
                }

                sys::sleep_ms(1).await;
            }

            count
        }
    }

    fn clear_rx(&mut self) {
        while self.read_byte().is_some() {}
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        for &b in s.as_bytes() {
            self.write_byte(b)?;
        }
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        for &byte in data {
            self.write_byte(byte)?;
        }
        Ok(())
    }
}
