#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(obamas::test::runner)]
#![reexport_test_harness_main = "_test"]

use obamas::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("ObamaS started successfully");

    #[cfg(test)]
    _test();

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    obamas::test::panic_handler(info)
}
