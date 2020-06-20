#![no_std]
#![cfg_attr(test, no_main)]
#![feature(const_fn)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::runner)]
#![reexport_test_harness_main = "_test"]

#[macro_use]
mod macros;

pub mod mem;
pub mod serial;
pub mod sync;
pub mod vga;

pub mod qemu;
pub mod test;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    _test();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test::panic_handler(info)
}
