#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(obamas::test::runner)]
#![reexport_test_harness_main = "_test"]

extern crate alloc;

use bootloader::BootInfo;
use obamas::println;
use x86_64::VirtAddr;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    obamas::init();

    println!("ObamaS booted successfully");

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { obamas::mem::paging::mapper(phys_offset) };
    let mut frame_allocator =
        unsafe { obamas::mem::paging::frame_allocator(&boot_info.memory_map) };

    obamas::mem::alloc::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    #[cfg(test)]
    _test();

    obamas::halt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    obamas::halt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    obamas::test::panic_handler(info)
}
