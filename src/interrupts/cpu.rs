use crate::gdt::DOUBLE_FAULT_IST_INDEX;
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

pub fn set_handlers(idt: &mut InterruptDescriptorTable) {
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
}

extern "x86-interrupt" fn double_fault_handler(sf: &mut InterruptStackFrame, _: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", sf);
}

extern "x86-interrupt" fn breakpoint_handler(sf: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", sf);
}

extern "x86-interrupt" fn page_fault_handler(
    sf: &mut InterruptStackFrame,
    err: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", err);
    println!("{:#?}", sf);
    crate::halt();
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn breakpoint() {
        x86_64::instructions::interrupts::int3();
    }
}
