
#[macro_export]
macro_rules! singleton {
    // ($type:ty) => {
    //     impl $type {
    //         pub fn init() {
    //             let _ = Self::get_mut();
    //         }
    //         pub fn get_mut() -> &'static mut Self {
    //             static mut INSTANCE: Option<$type> = None;
    //             unsafe {
    //                 if core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap().is_none() {
    //                     INSTANCE = Some(<$type>::new());
    //                 }
    //                 core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap().as_mut().unwrap()
    //             }
    //         }
    //     }
    // };
    // ($type:ty, $init_expr:expr) => {
    //     impl $type {
    //         pub fn init() {
    //             let _ = Self::get_mut();
    //         }
    //         pub fn get_mut() -> &'static mut Self {
    //             static mut INSTANCE: Option<$type> = None;
    //             unsafe {
    //                 if core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap().is_none() {
    //                     INSTANCE = Some($init_expr);
    //                 }
    //                 core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap().as_mut().unwrap()
    //             }
    //         }
    //     }
    // };
    ($type:ident { $($field:ident: $value:expr),* $(,)? }) => {
        impl $type {
            #[allow(unused)]
            pub fn init_instance() {
                let _ = Self::get_mut();
            }
            #[allow(unused)]
            pub fn get() -> &'static Self {
                Self::get_mut()
            }
            pub fn get_mut() -> &'static mut Self {
                static mut INSTANCE: Option<$type> = None;
                static mut INITIALIZED: bool = false;
                unsafe {
                    if !INITIALIZED {
                        INSTANCE = Some($type {
                            $($field: $value,)*
                        });
                        INITIALIZED = true;
                    }
                    core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked().as_mut().unwrap_unchecked()
//                    core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap().as_mut().unwrap()
                }
            }
        }
    };
}