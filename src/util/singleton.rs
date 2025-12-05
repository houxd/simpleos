
#[macro_export]
macro_rules! singleton {
    ($type:ident { $($field:ident: $value:expr),* $(,)? }) => {
        impl $type {
            // fn instance() -> &'static mut Option<$type> {
            //     static mut INSTANCE: Option<$type> = None;
            //     unsafe {
            //         if core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked().as_mut().is_none() {
            //             INSTANCE = Some($type {
            //                 $($field: $value,)*
            //             });
            //         }
            //         INSTANCE.as_mut().unwrap_unchecked()
            //     }
            // }

            pub fn ref_mut() -> &'static mut Self {
                static mut INSTANCE: Option<$type> = None;
                unsafe {
                    if core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked().as_mut().is_none() {
                        INSTANCE = Some($type {
                            $($field: $value,)*
                        });
                    }
                    core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked().as_mut().unwrap_unchecked()
                    // core::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap().as_mut().unwrap()
                }
            }
        }
    };
}