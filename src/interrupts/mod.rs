use crate::{gdt::DOUBLE_FAULT_IST_INDEX, sync::Lazy};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub fn init() {
    IDT.load();
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt
});

extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn breakpoint() {
        x86_64::instructions::interrupts::int3();
    }
}
