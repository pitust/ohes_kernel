use core::alloc::{GlobalAlloc, Layout};

use x86_64::structures::paging::PageTableFlags;

use crate::prelude::*;

ezy_static! { CANARIES, Vec<(VirtAddr, String, u64)>, vec![] }

pub fn add_canary(p: VirtAddr, s: String, size: u64) {
    let q = p.as_ptr::<u8>();
    unsafe {
        *(q as *mut u64) = 0xdeadbeef;
    }
    CANARIES.get().push((p, s.clone(), size));
}

extern "C" {
    #[link_name = "llvm.returnaddress"]
    fn return_address(level: i32) -> *const u8;
}
// AddressCleaner CleanGuardedAlloc

ezy_static! { RANGES, Vec<(Layout, String), &'static memory::allocator::WrapperAlloc>, Vec::new_in(&memory::allocator::WRAPPED_ALLOC) }
counter!(AddrLo);
counter!(ReentrancyGuard);
ezy_static! { _OLD_TMP_ENTER_LAYOUT, Option<Layout>, None }

pub struct CleaningAlloc {}
#[cfg_attr(address_cleaner, global_alloc)]
pub static CLEAN_ALLOC: CleaningAlloc = CleaningAlloc {};
impl CleaningAlloc {
    pub unsafe fn do_alloc(&self, layout: Layout) -> *mut u8 {
        // dprintln!("{:?} {:?}", layout, layout.align_to(16).unwrap());
        x86_64::instructions::interrupts::without_interrupts(|| {
            if layout.align() > 4096 {
                panic!("Align > 4k not supported");
            }
            if ReentrancyGuard::inc() != 1 {
                println!(
                    "=={}== ERROR: AddressCleaner memory-alloc-reentrancy at pc {:p}",
                    preempt::CURRENT_TASK.pid,
                    unsafe { return_address(0) }
                );
                println!(" => new layout is {:?}", layout);
                println!(
                    " => old layout is {:?}",
                    _OLD_TMP_ENTER_LAYOUT.get().unwrap()
                );
                println!("=={}== ABORTING", preempt::CURRENT_TASK.pid);
                panic!("[AddressCleaner abort]");
            }
            *_OLD_TMP_ENTER_LAYOUT.get() = Some(layout.clone());
            // AddrLo
            if AddrLo::get() == 0 {
                AddrLo::addn(0xf000000000);
            }
            // [GUARD ][ DATA ][GUARD ]
            // ^ old   ^       |
            //         \-start |
            //                 \-- _guardhi
            let start = AddrLo::addn(4096);
            let paddedsz = (layout.size() + 4095) / 4096 * 4096;
            let _end = AddrLo::addn(paddedsz);
            let _guardhiend = AddrLo::addn(4096);

            let p = ralloc::Allocator.alloc(layout.align_to(4096).unwrap());

            for i in 0..(paddedsz / 4096) {
                let j = i * 4096;
                memory::map_to(
                    VirtAddr::from_ptr(p.offset(j as isize)),
                    VirtAddr::new((start + j) as u64),
                    PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
                );
            }
            faster_rlibc::fastermemset(
                p.offset(layout.size() as isize),
                0x1badb0071badb007,
                paddedsz - layout.size(),
            );

            ReentrancyGuard::dec();
            _OLD_TMP_ENTER_LAYOUT.get().take();
            start as *mut u8
        })
    }
    pub unsafe fn do_dealloc(&self, ptr: *mut u8, layout: Layout) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if layout.align() > 4096 {
                panic!("Align > 4k not supported");
            }
            if ReentrancyGuard::inc() != 1 {
                println!(
                    "=={}== ERROR: AddressCleaner memory-alloc-reentrancy at pc {:p}",
                    preempt::CURRENT_TASK.pid,
                    unsafe { return_address(0) }
                );
                println!(" => new layout is {:?}", layout);
                println!(
                    " => old layout is {:?}",
                    _OLD_TMP_ENTER_LAYOUT.get().unwrap()
                );
                println!("=={}== ABORTING", preempt::CURRENT_TASK.pid);
                panic!("[AddressCleaner abort]");
            }
            *_OLD_TMP_ENTER_LAYOUT.get() = Some(layout.clone());
            // AddrLo
            if AddrLo::get() == 0 {
                panic!("Ahhh crap, AddressCleaner do_dealloc() called before do_alloc()...");
            }

            // [GUARD ][ DATA ][GUARD ]
            // ^ old   ^       |
            //         \-start |
            //                 \-- _guardhi
            let paddedsz = (layout.size() + 4095) / 4096 * 4096;

            for n in 0..((paddedsz - layout.size()) / 8) {
                // (n * 8) as isize
                if unsafe {
                    (ptr as *mut u64)
                        .offset((layout.size() + (n * 8)) as isize)
                        .read()
                } != 0x1badb0071badb007
                {
                    println!(
                        "=={}== ERROR: AddressCleaner shadow-space-write (checked at pc {:p})",
                        preempt::CURRENT_TASK.pid,
                        unsafe { return_address(0) }
                    );
                    println!(" => memory layout is {:?}", layout);
                    println!(
                        " => write at {:p}",
                        ptr.offset((layout.size() + (n * 8)) as isize)
                    );
                    // show shadow space:
                    let mut ptr = ptr.offset((layout.size() + (n * 8)) as isize);
                    if ptr as u64 & 0xf == 0x8 {
                        ptr = ptr.offset(-8);
                    }
                    ptr = ptr.offset(-32);
                    print!("Data:");
                    let dat: &[u8] = &[0x07, 0xb0, 0xad, 0x1b];
                    for i in 0..64 {
                        if i & 0xf == 0 {
                            print!("\n{:p}:", ptr.offset(i));
                        }
                        let val = unsafe { ptr.read() };
                        if dat[(i & 3) as usize] == val {
                            println!("  {:#016x?} ", val);
                        } else {
                            println!(" [{:#016x?}]", val);
                        }
                    }
                }
            }

            for i in 0..(paddedsz / 4096) {
                let j = i * 4096;
                memory::munmap(VirtAddr::from_ptr(ptr.offset(j as isize)));
            }

            ralloc::Allocator.dealloc(
                memory::convpm(ptr) as *mut u8,
                layout.align_to(4096).unwrap(),
            );

            ReentrancyGuard::dec();
            _OLD_TMP_ENTER_LAYOUT.get().take();
            // start as *mut u8
        })
    }
}
unsafe impl core::alloc::GlobalAlloc for CleaningAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return self.do_alloc(layout);
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        return self.do_dealloc(ptr, layout);
    }
}

//// EndAddressCleaner CleanGuardedAlloc

pub fn stk_chk() {
    for c in CANARIES.get() {
        let q = c.0.as_ptr::<u8>();
        let stka = unsafe { *(q as *mut u64) };
        if stka != 0xdeadbeef {
            println!(
                "=={}== ERROR: AddressCleaner stack-overflow on address {:p} at pc {:p} stack {}",
                preempt::CURRENT_TASK.pid,
                q,
                unsafe { return_address(0) },
                c.1
            );
            println!("SMASHED of size 8 at {:p} (now is {:#x?})", q, stka);
            println!("=={}== ABORTING", preempt::CURRENT_TASK.pid);
        }
    }
}
