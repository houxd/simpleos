#[macro_export]
macro_rules! device {
    ($type:ident { $($field:ident: $field_type:ty = $value:expr),* $(,)? }) => {

        // 生成结构体定义
        pub struct $type {
            $(pub $field: $field_type),*
        }

        // 生成单例结构体
        crate::singleton!($type { $($field: $value),* });

        // 生成每个字段的快捷访问方法
        impl $type {
            $(
                #[inline]
                pub fn $field() -> &'static mut $field_type {
                    &mut Self::mut_ref().$field
                }
            )*
        }

        // 生成初始化方法
        impl $type {
            pub fn device_init() -> anyhow::Result<()>{
                $(
                    Self::mut_ref().$field.driver_init()?;
                )*
                Ok(())
            }
            pub fn device_deinit() -> anyhow::Result<()>{
                $(
                    Self::mut_ref().$field.driver_deinit()?;
                )*
                Ok(())
            }
        }
    };
}
