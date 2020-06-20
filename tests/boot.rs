#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(obamas::test::runner)]
#![reexport_test_harness_main = "_test"]

use obamas::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    _test();
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    obamas::test::panic_handler(info)
}

#[test_case]
fn println_single() {
    println!("Hello, World!");
}
