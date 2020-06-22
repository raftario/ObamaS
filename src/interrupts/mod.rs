mod cpu;
mod hw;

use crate::sync::Lazy;
use x86_64::structures::idt::InterruptDescriptorTable;

pub fn init() {
    IDT.load();
    hw::init();
}

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    cpu::set_handlers(&mut idt);
    hw::set_handlers(&mut idt);
    idt
});
