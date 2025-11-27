use crate::{console::ConsoleDriver, driver::systick::SysTickDriver};

pub trait Device {
    fn default_console(&self) -> &'static mut dyn ConsoleDriver;
    fn default_systick(&self) -> &'static mut dyn SysTickDriver;
    fn init(&self);
}

/// 设备表宏定义
/// 用于生成设备的单例结构体和板级设备初始化/反初始化方法
/// 语法:
/// device_table!(BoardName, {
///     Device1: DriverType1 = value1,
///     Device2: DriverType2 = value2,
///     ...
/// });
#[macro_export]
macro_rules! device_table {
    ($board_name:ident, { $($device_name:ident: $driver_type:ty = $value:expr),* $(,)? }) => {
        $(
            // 生成设备结构体定义
            pub struct $device_name {
                dev: $driver_type,
            }

            // 生成每个设备的单例结构体
            crate::singleton!($device_name { dev: $value });

            // 生成每个设备的快捷访问方法
            impl $device_name {
                #[inline]
                pub fn dev() -> &'static mut $driver_type {
                    &mut Self::ref_mut().dev
                }
            }
        )*

        // 生成板级设备初始化和反初始化方法
        impl $board_name {
            pub fn devices_init() -> anyhow::Result<()> {
                $(
                    $device_name::dev().driver_init()?;
                )*
                Ok(())
            }
            pub fn devices_deinit() -> anyhow::Result<()> {
                $(
                    $device_name::dev().driver_deinit()?;
                )*
                Ok(())
            }
        }

    };
}
