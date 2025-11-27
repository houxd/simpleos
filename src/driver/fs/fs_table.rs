

#[macro_export]
macro_rules! fs_table {
    ($board_name:ident, { $($fs_name:ident: $fs_type:ty = $value:expr),* $(,)? }) => {
        $(
            // 生成文件系统结构体定义
            pub struct $fs_name {
                fs: $fs_type,
            }

            // 生成每个文件系统的单例结构体
            crate::singleton!($fs_name { fs: $value });

            // 生成每个文件系统的快捷访问方法
            impl $fs_name {
                #[inline]
                pub fn fs() -> &'static mut $fs_type {
                    &mut Self::ref_mut().fs
                }
            }
        )*

        // 生成板级文件系统初始化和反初始化方法
        struct $board_name;
        impl $board_name {
            pub fn fs_init() -> anyhow::Result<()> {
                $(
                    $fs_name::fs().fs_init()?;
                )*
                Ok(())
            }
            pub fn fs_deinit() -> anyhow::Result<()> {
                $(
                    $fs_name::fs().fs_deinit()?;
                )*
                Ok(())
            }
        }

    };
}