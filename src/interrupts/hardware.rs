use crate::sync::{Lazy, Mutex};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

pub fn set_handlers(idt: &mut InterruptDescriptorTable) {
    idt[InterruptIndex::Timer.into()].set_handler_fn(timer_handler);
    idt[InterruptIndex::Keyboard.into()].set_handler_fn(keyboard_handler);
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl From<InterruptIndex> for usize {
    fn from(i: InterruptIndex) -> Self {
        i as u8 as Self
    }
}

extern "x86-interrupt" fn timer_handler(_: &mut InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_handler(_: &mut InterruptStackFrame) {
    static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>>> = Lazy::new(|| {
        Mutex::new(Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::Ignore,
        ))
    });

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
