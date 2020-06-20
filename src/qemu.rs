use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit(exit_code: ExitCode) {
    unsafe { Port::new(0xF4).write(exit_code as u32) }
}
