#![no_std]
#![cfg_attr(test, no_main)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::runner)]
#![reexport_test_harness_main = "_test"]

extern crate alloc;

#[macro_use]
mod macros;

pub mod gdt;
pub mod interrupts;
pub mod mem;
pub mod rand;
pub mod serial;
pub mod sync;
pub mod time;
pub mod vga;

pub mod qemu;
pub mod test;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
    init();

    let phys_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { mem::paging::mapper(phys_offset) };
    let mut frame_allocator = unsafe { mem::paging::frame_allocator(&boot_info.memory_map) };

    mem::alloc::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    _test();
    halt()
}

pub fn init() {
    gdt::init();
    interrupts::init();
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test::panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
