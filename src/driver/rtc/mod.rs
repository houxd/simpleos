use crate::driver::Driver;
use anyhow::Result;
use chrono::NaiveDateTime;

pub trait RtcDriver: Driver {
    fn rtc_read_datetime(&mut self) -> Result<NaiveDateTime>;
    fn rtc_write_datetime(&mut self, dt: &NaiveDateTime) -> Result<()>;
    fn rtc_get_timestamp_sec(&mut self) -> Result<i64> {
        Ok(self.rtc_read_datetime()?.and_utc().timestamp())
    }
}

pub mod hym8563;
