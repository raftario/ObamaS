use crate::sync::Lazy;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub fn init() {
    GDT.0.load();
    unsafe {
        x86_64::instructions::segmentation::set_cs(GDT.1.code);
        x86_64::instructions::tables::load_tss(GDT.1.tss);
    }
}

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (
        gdt,
        Selectors {
            code: code_selector,
            tss: tss_selector,
        },
    )
});

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
    };
    tss
});
