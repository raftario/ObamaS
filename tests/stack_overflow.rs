#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use obamas::{mem::Volatile, qemu, s1print, s1println, sync::Lazy};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static TEST_IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
        idt.double_fault
            .set_handler_fn(test_double_fault_handler)
            .set_stack_index(obamas::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
});

extern "x86-interrupt" fn test_double_fault_handler(_: &mut InterruptStackFrame, _: u64) -> ! {
    s1println!("ok");
    qemu::exit(qemu::ExitCode::Success);
    obamas::halt();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    s1print!("{} ... ", module_path!());

    obamas::gdt::init();
    TEST_IDT.load();

    stack_overflow();

    panic!("Execution after stack overflow")
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    obamas::test::panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    Volatile::new(0).read();
}
