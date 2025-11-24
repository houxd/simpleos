#[macro_export]
macro_rules! print {
    () => {{
        $crate::console::Console::get_mut().io.csl_flush();
    }};
    ($($arg:tt)*) => {{
        let formatted = alloc::format!($($arg)*);
        $crate::console::Console::get_mut().io.write(formatted.as_bytes());
        $crate::console::Console::get_mut().io.csl_flush();
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::console::Console::get_mut().io.write(b"\r\n");
        $crate::console::Console::get_mut().io.csl_flush();
    }};
    ($($arg:tt)*) => {{
        let formatted = alloc::format!($($arg)*);
        $crate::console::Console::get_mut().io.write(formatted.as_bytes());
        $crate::console::Console::get_mut().io.write(b"\r\n");
        $crate::console::Console::get_mut().io.csl_flush();
    }};
}