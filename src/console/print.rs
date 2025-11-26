#[macro_export]
macro_rules! print {
    () => {{
        $crate::console::Console::device().console_flush();
    }};
    ($($arg:tt)*) => {{
        let formatted = alloc::format!($($arg)*);
        $crate::console::Console::device().console_write(formatted.as_bytes());
        $crate::console::Console::device().console_flush();
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::console::Console::device().console_write(b"\r\n");
        $crate::console::Console::device().console_flush();
    }};
    ($($arg:tt)*) => {{
        let formatted = alloc::format!($($arg)*);
        $crate::console::Console::device().console_write(formatted.as_bytes());
        $crate::console::Console::device().console_write(b"\r\n");
        $crate::console::Console::device().console_flush();
    }};
}