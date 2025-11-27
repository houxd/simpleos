#[macro_export]
macro_rules! print {
    () => {{
        if $crate::SimpleOs::is_initialized() {
            $crate::$SimpleOs::device().default_console().console_flush();
        }
    }};
    ($($arg:tt)*) => {{
        if $crate::SimpleOs::is_initialized() {
            let formatted = alloc::format!($($arg)*);
            $crate::SimpleOs::device().default_console().console_write(formatted.as_bytes());
            $crate::SimpleOs::device().default_console().console_flush();
        }
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        if $crate::SimpleOs::is_initialized() {
            $crate::SimpleOs::device().default_console().console_write(b"\r\n");
            $crate::SimpleOs::device().default_console().console_flush();
        }
    }};
    ($($arg:tt)*) => {{
        if $crate::SimpleOs::is_initialized() {
            let formatted = alloc::format!($($arg)*);
            $crate::SimpleOs::device().default_console().console_write(formatted.as_bytes());
            $crate::SimpleOs::device().default_console().console_write(b"\r\n");
            $crate::SimpleOs::device().default_console().console_flush();
        }
    }};
}
