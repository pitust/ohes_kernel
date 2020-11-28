use crate::println;
use alloc::{alloc::Layout, vec::Vec};
// use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use crate::shittymutex::Mutex;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use multiboot2::{MemoryArea, MemoryAreaType, MemoryMapTag};
use x86_64::structures::paging::page_table::PageTable;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, MapperAllSizes, OffsetPageTable, Page, PageTableFlags, PhysFrame,
    Size4KiB,
};
use x86_64::{registers::control::Cr3, structures::paging::page_table::PageTableEntry};
use x86_64::{PhysAddr, VirtAddr};
// pub static PHBASE: AtomicUsize = AtomicUsize::new(0);
pub mod allocator;
pub fn munmap(area: VirtAddr) {
    let u = crate::memory::get_mapper()
        .unmap(Page::<Size4KiB>::containing_address(area))
        .unwrap()
        .1;
    u.flush();
}
pub fn try_munmap(area: VirtAddr) {
    match crate::memory::get_mapper().unmap(Page::<Size4KiB>::containing_address(area)) {
        Ok(o) => {
            o.1.flush();
        }
        Err(_) => {}
    }
}
pub fn map_to(from: VirtAddr, to: VirtAddr, flags: PageTableFlags) {
    let frame = PhysFrame::<Size4KiB>::containing_address(crate::memory::translate(from).unwrap());
    let flags = PageTableFlags::PRESENT | flags;
    println!("map {:?} -> {:?}", from, to);
    try_munmap(to);
    let map_to_result = unsafe {
        crate::memory::get_mapper().map_to(
            Page::containing_address(to),
            frame,
            flags,
            &mut crate::memory::FRAME_ALLOC.get().unwrap(),
        )
    };
    map_to_result.expect("map_to failed").flush();
}
#[macro_export]
macro_rules! phmem_offset {
    () => {
        x86_64::VirtAddr::new(0 as u64)
    };
}

pub fn get_mapper() -> OffsetPageTable<'static> {
    unsafe { OffsetPageTable::new(get_l4(), phmem_offset!()) }
}
pub fn get_l4() -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phmem_offset!() + phys.as_u64();
    let ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *ptr }
}

pub fn translate(addr: VirtAddr) -> Option<PhysAddr> {
    get_mapper().translate_addr(addr)
}
fn get_entry_for(addr: VirtAddr) -> Option<&'static PageTableEntry> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;
    let mut last_entry = unsafe { &*(0 as *const PageTableEntry) };
    // traverse the multi-level page table
    for &index in &table_indexes {
        // convert the frame into a page table reference
        let virt = phmem_offset!() + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => return Some(entry),
        };
        last_entry = entry;
    }

    // calculate the physical address by adding the page offset
    Some(last_entry)
}
pub fn get_flags_for(va: VirtAddr) -> Option<PageTableFlags> {
    match get_entry_for(va) {
        Some(f) => Some(f.flags()),
        None => None,
    }
}

pub fn create_example_mapping(page: Page, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xfd000000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_to_result = unsafe { get_mapper().map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

static NEXT: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug, Copy, Clone)]
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMapTag,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMapTag) -> Self {
        BootInfoFrameAllocator { memory_map }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.memory_areas();
        let usable_regions = regions.filter(|r| r.typ() == MemoryAreaType::Available);
        let addr_ranges = usable_regions.map(|r| r.start_address()..r.end_address());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
    pub fn show(&self) {
        let x: Vec<PhysFrame<Size4KiB>> = self.usable_frames().collect();
        println!("Frames: {:#?}", x);
    }
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self
            .usable_frames()
            .nth(NEXT.fetch_add(1, Ordering::Relaxed));
        frame
    }
}
lazy_static! {
    pub static ref FRAME_ALLOC: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);
}

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe {
        let lsz = core::mem::size_of::<usize>();
        let data = crate::memory::allocator::WRAPPED_ALLOC
            .do_alloc(Layout::from_size_align(size + lsz, 8).unwrap());
        if data == 0 as *mut u8 {
            panic!("Bad alloc!");
        }
        (*(data as *mut usize)) = size + lsz;
        data.offset(lsz as isize)
    }
}

#[no_mangle]
pub extern "C" fn free(el: *mut u8) {
    unsafe {
        crate::memory::allocator::WRAPPED_ALLOC.dealloc(
            el.offset(-8),
            Layout::from_size_align(*(el.offset(-8) as *const usize), 8).unwrap(),
        )
    };
}

pub fn mpage() -> *mut u8 {
    unsafe {
        let data = crate::memory::allocator::WRAPPED_ALLOC
            .do_alloc(Layout::from_size_align(4096, 4096).unwrap());
        if data == 0 as *mut u8 {
            panic!("Bad alloc!");
        }
        data
    }
}

pub fn fpage(el: *mut u8) {
    unsafe {
        crate::memory::allocator::WRAPPED_ALLOC
            .dealloc(el, Layout::from_size_align(4096, 4096).unwrap())
    };
}
