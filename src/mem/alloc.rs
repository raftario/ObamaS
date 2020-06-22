use crate::sync::Mutex;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

#[global_allocator]
static ALLOCATOR: Mutex<BlockAlloc> = Mutex::new(BlockAlloc::new());

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct Block {
    next: Option<&'static mut Block>,
}
pub struct BlockAlloc {
    heads: [Option<&'static mut Block>; BLOCK_SIZES.len()],
    fallback: linked_list_allocator::Heap,
}

impl BlockAlloc {
    pub const fn new() -> Self {
        Self {
            heads: [None; BLOCK_SIZES.len()],
            fallback: linked_list_allocator::Heap::empty(),
        }
    }
    unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Mutex<BlockAlloc> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match head(&layout) {
            Some(idx) => match allocator.heads[idx].take() {
                Some(block) => {
                    allocator.heads[idx] = block.next.take();
                    block as *mut Block as *mut u8
                }
                None => {
                    let block_size = BLOCK_SIZES[idx];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    allocator.fallback_alloc(layout)
                }
            },
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match head(&layout) {
            Some(idx) => {
                assert!(mem::size_of::<Block>() <= BLOCK_SIZES[idx]);
                assert!(mem::align_of::<Block>() <= BLOCK_SIZES[idx]);

                let new_block = Block {
                    next: allocator.heads[idx].take(),
                };
                #[allow(clippy::cast_ptr_alignment)]
                let mew_block_ptr = ptr as *mut Block;
                mew_block_ptr.write(new_block);
                allocator.heads[idx] = Some(&mut *mew_block_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback.deallocate(ptr, layout);
            }
        }
    }
}

fn head(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
