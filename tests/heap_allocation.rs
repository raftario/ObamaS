#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(obamas::test::runner)]
#![reexport_test_harness_main = "_test"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use bootloader::BootInfo;
use x86_64::VirtAddr;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { obamas::mem::paging::mapper(phys_offset) };
    let mut frame_allocator =
        unsafe { obamas::mem::paging::frame_allocator(&boot_info.memory_map) };

    obamas::mem::alloc::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    _test();

    obamas::halt();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    obamas::test::panic_handler(info)
}

#[test_case]
fn boxes() {
    let box1 = Box::new(21);
    let box2 = Box::new(12);
    assert_eq!(*box1, 21);
    assert_eq!(*box2, 12);
}

#[test_case]
fn vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn fill() {
    for i in 0..obamas::mem::alloc::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn fill_long_lived() {
    let long = Box::new(2112);
    for i in 0..obamas::mem::alloc::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long, 2112);
}
