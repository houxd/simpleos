#[macro_export]
macro_rules! print {
    () => {{
        if $crate::sys::SimpleOs::is_initialized() {
            $crate::sys::SimpleOs::console().console_flush();
        }
    }};
    ($($arg:tt)*) => {{
        if $crate::sys::SimpleOs::is_initialized() {
            let formatted = alloc::format!($($arg)*);
            $crate::sys::SimpleOs::console().console_write(formatted.as_bytes());
            $crate::sys::SimpleOs::console().console_flush();
        }
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        if $crate::sys::SimpleOs::is_initialized() {
            $crate::sys::SimpleOs::console().console_write(b"\r\n");
            $crate::sys::SimpleOs::console().console_flush();
        }
    }};
    ($($arg:tt)*) => {{
        if $crate::sys::SimpleOs::is_initialized() {
            let formatted = alloc::format!($($arg)*);
            $crate::sys::SimpleOs::console().console_write(formatted.as_bytes());
            $crate::sys::SimpleOs::console().console_write(b"\r\n");
            $crate::sys::SimpleOs::console().console_flush();
        }
    }};
}
