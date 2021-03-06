use crate::{dprintln, println};
use alloc::{alloc::Layout, vec::Vec};
use allocator::CUR_ADDR_PUB;
use multiboot::information::{MemoryMapIter, MemoryType};
// use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use crate::shittymutex::Mutex;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
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
    if translate(to).is_some() {
        return;
    }
    let frame = PhysFrame::<Size4KiB>::containing_address(crate::memory::translate(from).unwrap());
    let flags = PageTableFlags::PRESENT | flags;
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
pub fn convig(x: u64) -> u64 {
    crate::memory::translate(VirtAddr::new(x as u64))
        .unwrap()
        .as_u64()
}
pub fn convpc<T>(x: *const T) -> u64 {
    crate::memory::translate(VirtAddr::new(x as u64))
        .unwrap()
        .as_u64()
}
pub fn convpm<T>(x: *mut T) -> u64 {
    crate::memory::translate(VirtAddr::new(x as u64))
        .unwrap()
        .as_u64()
}
pub fn ispm<T>(x: *mut T) -> bool {
    crate::memory::translate(VirtAddr::new(x as u64)).is_some()
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

static NEXT: AtomicUsize = AtomicUsize::new(0x100000);
#[derive(Copy, Clone)]
pub struct BootInfoFrameAllocator {
    end: u64,
}

extern "C" {
    pub static es: u8;
    pub static esz: u8;
    pub static ee: u8;
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(size: u32) -> Self {
        BootInfoFrameAllocator {
            end: 0x100000 + (size as u64 * 0x400),
        }
    }
}
unsafe impl Send for BootInfoFrameAllocator {}
unsafe impl Sync for BootInfoFrameAllocator {}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        loop {
            let a = NEXT.fetch_add(4096, Ordering::Relaxed) as u64;
            assert!(a < self.end);
            if unsafe { (&ee as *const u8 as u64) > a } {
                continue;
            }
            return Some(PhysFrame::from_start_address(PhysAddr::new(a)).unwrap());
        }
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
        crate::memory::allocator::WRAPPED_ALLOC.do_dealloc(
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
            .do_dealloc(el, Layout::from_size_align(4096, 4096).unwrap())
    };
}
#[no_mangle]
pub extern "C" fn brk(to: *const u8) -> *mut u8 {
    // yeee
    if to == 0 as *const u8 {
        return allocator::CUR_ADDR_PUB.load(Ordering::Relaxed) as *mut u8;
    }
    let data = allocator::CUR_ADDR_PUB.load(Ordering::Relaxed);
    allocator::expand_by((to as u64) - data);
    to as *mut u8
}
