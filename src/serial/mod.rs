use crate::sync::{Lazy, Mutex};
use core::fmt;
use uart_16550::SerialPort;

#[doc(hidden)]
pub fn _print1(args: fmt::Arguments) {
    use fmt::Write;
    SERIAL1.lock().write_fmt(args).unwrap();
}

pub static SERIAL1: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    Mutex::new(serial_port)
});
