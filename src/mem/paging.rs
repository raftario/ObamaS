use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, Mapper, OffsetPageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// # Safety
/// An invalid offset will just completely fuck up paging
pub unsafe fn mapper(phys_offset: VirtAddr) -> impl Mapper<Size4KiB> + 'static {
    let (frame, _) = Cr3::read();

    let phys = frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let l4_table = &mut *virt.as_mut_ptr();

    OffsetPageTable::new(l4_table, phys_offset)
}

/// # Safety
/// An invalid memory map will just completely fuck up paging
pub unsafe fn frame_allocator(memory_map: &'static MemoryMap) -> impl FrameAllocator<Size4KiB> {
    BootInfoFrameAllocator {
        memory_map,
        next: 0,
    }
}

struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
