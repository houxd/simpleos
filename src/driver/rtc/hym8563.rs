// #![allow(unused)]

use crate::{
    driver::{i2c::I2cDriver, rtc::RtcDriver, Driver},
    util,
};
use alloc::boxed::Box;
use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

// HYM8563T I2C 地址和寄存器定义
const HYM8563_ADDR: u16 = 0x51 << 1;

// 寄存器地址
const REG_CTRL_STATUS1: u8 = 0x00;
const REG_CTRL_STATUS2: u8 = 0x01;
const REG_SECONDS: u8 = 0x02;
// const REG_MINUTES: u8 = 0x03;
// const REG_HOURS: u8 = 0x04;
// const REG_DAYS: u8 = 0x05;
// const REG_WEEKDAYS: u8 = 0x06;
// const REG_MONTHS: u8 = 0x07;
// const REG_YEARS: u8 = 0x08;

// 控制位定义
const STOP_BIT: u8 = 0x20;
// const TEST_BIT: u8 = 0x08;
const RESET_BIT: u8 = 0x80;
const VL_BIT: u8 = 0x80; // 数据有效位

pub struct Hym8563 {
    pub i2c: Box<&'static mut dyn I2cDriver>,
}

impl Hym8563 {
    fn write_register(&mut self, reg: u8, data: u8) -> Result<()> {
        let buffer = [reg, data];
        self.i2c.i2c_write(HYM8563_ADDR, buffer.as_ref())
    }

    fn burst_write(&mut self, start_reg: u8, data: &[u8]) -> Result<()> {
        let mut buffer = [0u8; 8]; // 最多8字节
        if data.len() > 7 {
            return Err(anyhow!("HYM8563 EX WSIZE"));
        }

        buffer[0] = start_reg;
        buffer[1..=data.len()].copy_from_slice(data);

        self.i2c
            .i2c_write(HYM8563_ADDR, buffer[..=data.len()].as_ref())?;
        Ok(())
    }

    fn read_register(&mut self, reg: u8) -> Result<u8> {
        // 先发送寄存器地址
        let reg_buf = [reg];
        self.i2c.i2c_write(HYM8563_ADDR, reg_buf.as_ref())?;

        // 再读取数据
        let mut buffer = [0u8; 1];
        self.i2c.i2c_read(HYM8563_ADDR, buffer.as_mut())?;

        Ok(buffer[0])
    }

    fn burst_read(&mut self, start_reg: u8, buffer: &mut [u8]) -> Result<()> {
        // 先发送寄存器地址
        let reg_buf = [start_reg];
        self.i2c.i2c_write(HYM8563_ADDR, reg_buf.as_ref())?;

        // 再读取数据
        self.i2c.i2c_read(HYM8563_ADDR, buffer)?;

        Ok(())
    }

    pub fn is_running(&mut self) -> Result<bool> {
        let status = self.read_register(REG_CTRL_STATUS1)?;
        Ok((status & STOP_BIT) == 0)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.write_register(REG_CTRL_STATUS2, RESET_BIT)
    }

    fn init(&mut self) -> Result<()> {
        // 复位 RTC
        self.reset()?;

        // 等待一段时间让复位完成, 在实际应用中可能需要添加延时
        for _ in 0..10000 {}

        // 清除控制寄存器1，确保RTC启动
        self.write_register(REG_CTRL_STATUS1, 0x00)?;

        // 清除控制寄存器2，禁用中断等
        self.write_register(REG_CTRL_STATUS2, 0x00)?;

        Ok(())
    }
    fn deinit(&mut self) -> Result<()> {
        // 停止 RTC
        let mut status = self.read_register(REG_CTRL_STATUS1)?;
        status |= STOP_BIT;
        self.write_register(REG_CTRL_STATUS1, status)
    }
    fn get_datetime(&mut self) -> Result<NaiveDateTime> {
        let mut buffer = [0u8; 7];
        self.burst_read(REG_SECONDS, &mut buffer)?;

        // 检查 VL (Valid Low) 位
        if buffer[0] & VL_BIT != 0 {
            return Err(anyhow!("RTC data invalid (VL bit set)"));
        }

        let second = util::bcd_to_dec(buffer[0] & 0x7F)?;
        let minute = util::bcd_to_dec(buffer[1] & 0x7F)?;
        let hour = util::bcd_to_dec(buffer[2] & 0x3F)?;
        let day = util::bcd_to_dec(buffer[3] & 0x3F)?;
        let month = util::bcd_to_dec(buffer[5] & 0x1F)?;
        let year = 2000 + util::bcd_to_dec(buffer[6])? as i32;
        // 验证时间范围
        if second > 59
            || minute > 59
            || hour > 23
            || day == 0
            || day > 31
            || month == 0
            || month > 12
        {
            return Err(anyhow!("Invalid time"));
        }

        let naive_date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
            .ok_or(anyhow!("Invalid date"))?;
        let naive_time = naive_date
            .and_hms_opt(hour as u32, minute as u32, second as u32)
            .ok_or(anyhow!("Invalid time"))?;

        Ok(naive_time)
    }

    fn set_datetime(&mut self, naive: &NaiveDateTime) -> Result<()> {
        let second = util::dec_to_bcd(naive.second() as u8)?;
        let minute = util::dec_to_bcd(naive.minute() as u8)?;
        let hour = util::dec_to_bcd(naive.hour() as u8)?;
        let day = util::dec_to_bcd(naive.day() as u8)?;
        let weekday = naive.weekday().num_days_from_sunday() as u8; // 0-6, 0为周日
        let month = util::dec_to_bcd(naive.month() as u8)?;
        let year = util::dec_to_bcd((naive.year() % 100) as u8)?;

        let buffer = [
            second & 0x7F,
            minute & 0x7F,
            hour & 0x3F,
            day & 0x3F,
            weekday & 0x07,
            month & 0x1F,
            year,
        ];

        self.burst_write(REG_SECONDS, &buffer)
    }
}

impl Driver for Hym8563 {
    fn driver_init(&mut self) -> Result<()> {
        self.init()
    }

    fn driver_deinit(&mut self) -> Result<()> {
        self.deinit()
    }
}

impl RtcDriver for Hym8563 {
    fn rtc_read_datetime(&mut self) -> Result<NaiveDateTime> {
        self.get_datetime()
    }

    fn rtc_write_datetime(&mut self, naive: &NaiveDateTime) -> Result<()> {
        self.set_datetime(naive)
    }
}
