use crate::{print, println};
use core::{ptr::NonNull, alloc::{GlobalAlloc, AllocRef, Layout}};
use core::ptr::null_mut;
use linked_list_allocator::{Heap, LockedHeap};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;
pub struct WrapperAlloc {}
#[global_allocator]
pub static WRAPPED_ALLOC: WrapperAlloc = WrapperAlloc {};
impl WrapperAlloc {
    pub unsafe fn do_alloc(&self, layout: Layout) -> *mut u8 {
        let l2 = layout.clone();
        let mut d = ALLOCATOR.get().allocate_first_fit(layout.clone());
        while d.is_err() {
            expand_by(layout.size() as u64 * 4);
            d = ALLOCATOR.get().allocate_first_fit(l2.clone());
        }
        return d.unwrap().as_ptr();
    }
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.get().deallocate(NonNull::new(ptr).unwrap(), layout);
    }
}
unsafe impl core::alloc::GlobalAlloc for WrapperAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return self.do_alloc(layout);
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.get().deallocate(NonNull::new(ptr).unwrap(), layout);
    }
}
pub static ALLOCATOR: crate::shittymutex::Mutex<Heap> = crate::shittymutex::Mutex::new(Heap::empty());
pub const HEAP_START: usize = 0x100000000;
pub const HEAP_SIZE: usize = 4 * 1024;
pub const COW_PAGE: PageTableFlags = PageTableFlags::BIT_10;
pub const STACK_PAGE: PageTableFlags = PageTableFlags::BIT_11;
static CUR_ADDR: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new((HEAP_START + HEAP_SIZE) as u64);

pub fn expand_by(size: u64) {
    let size = ((size + 4095) / 4096) * 4096;
    let num = CUR_ADDR.fetch_add(size, core::sync::atomic::Ordering::Relaxed);
    expand_ram(num, size).expect("Failed expanding RAM");
}
pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let mut frame_alloc = crate::memory::FRAME_ALLOC
        .get()
        .expect("A frame allocator was not made yet");

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            super::get_mapper()
                .map_to(page, frame, flags, &mut frame_alloc)?
                .flush();
        };
    }

    unsafe {
        ALLOCATOR.get().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

fn expand_ram(from: u64, size: u64) -> Result<(), MapToError<Size4KiB>> {
    let mut frame_alloc = crate::memory::FRAME_ALLOC
        .get()
        .expect("A frame allocator was not made yet");
    let page_range = {
        let heap_start = VirtAddr::new(from as u64) + 1u64;
        let heap_end = heap_start + size - 4u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range {
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            super::get_mapper()
                .map_to(page, frame, flags, &mut frame_alloc)?
                .flush();
        };
    }

    unsafe {
        ALLOCATOR.get().extend(size as usize);
    }

    Ok(())
}
