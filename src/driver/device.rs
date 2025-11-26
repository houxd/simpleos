#[macro_export]
macro_rules! device {
    ($board_name:ident { $($device_name:ident: $driver_type:ty = $value:expr),* $(,)? }) => {
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
                    &mut Self::mut_ref().dev
                }
            }
        )*

        // 生成板级设备初始化和反初始化方法
        struct $board_name;
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

