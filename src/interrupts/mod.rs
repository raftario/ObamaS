mod cpu;
mod hardware;

use crate::sync::Lazy;
use x86_64::structures::idt::InterruptDescriptorTable;

pub fn init() {
    IDT.load();
    unsafe { hardware::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    cpu::set_handlers(&mut idt);
    hardware::set_handlers(&mut idt);
    idt
});
