#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! s1print {
    ($($arg:tt)*) => ($crate::serial::_print1(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! s1println {
    () => ($crate::s1print!("\n"));
    ($fmt:expr) => ($crate::s1print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::s1print!(concat!($fmt, "\n"), $($arg)*));
}
