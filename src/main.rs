use crate::prelude::*;
use crate::task::{simple_executor::SimpleExecutor, Task};
use core::panic::PanicInfo;
use multiboot::information::{MemoryManagement, Multiboot};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{PhysFrame, Size4KiB},
    VirtAddr,
};
use xmas_elf::{self, symbol_table::Entry};

pub fn i2s(n: u32) -> alloc::string::String {
    n.to_string()
}
#[macro_export]
macro_rules! add_fn {
    ($fcn: ident) => {
        crate::backtrace::mark(
            $fcn as *const (),
            &(alloc::string::String::from(stringify!($fcn))
                + " @ "
                + file!()
                + ":"
                + &$crate::i2s(line!())),
        );
    };
}
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} test(s)", tests.len());
    io::Printer.set_color(255, 255, 255);
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_test() {
    testing::test_header("Trivial Test");
    assert_eq!(1, 1);
    testing::test_ok();
}

#[test_case]
fn int3_no_crash() {
    testing::test_header("Int3 no crash");
    x86_64::instructions::interrupts::int3();
    testing::test_ok();
}

#[test_case]
fn no_crash_alloc() {
    testing::test_header("No crash on alloc");
    let _ = Box::new(41);
    testing::test_ok();
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    io::Printer.set_color(255, 0, 0);
    println!("--------------- Kernel Panic (not syncing) ---------------");
    println!("pid: {}", preempt::CURRENT_TASK.pid);
    println!("info: {}", info);
    println!("] stack checking...");
    stack_canaries::stk_chk();
    println!("] stacks checked");
    // unwind::backtrace();
    exiting::exit_fail();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn forkp() -> (PhysFrame, Cr3Flags) {
    // let's get my pages
    let old_pt = crate::memory::get_l4();
    let new_pt = crate::memory::mpage();
    unsafe {
        crate::faster_rlibc::fastermemcpy(
            new_pt,
            old_pt as *mut x86_64::structures::paging::PageTable as *const u8,
            2048,
        );
        crate::faster_rlibc::fastermemset(new_pt.offset(2048), 0, 2048);
    }
    let flags = Cr3::read().1;
    let q = PhysFrame::containing_address(
        crate::memory::translate(VirtAddr::from_ptr(new_pt)).unwrap(),
    );
    unsafe {
        Cr3::write(
            PhysFrame::containing_address(
                crate::memory::translate(VirtAddr::from_ptr(new_pt)).unwrap(),
            ),
            flags,
        );
    }
    (q, flags)
}

struct BadMMAN;
impl MemoryManagement for BadMMAN {
    unsafe fn paddr_to_slice(
        &self,
        addr: multiboot::information::PAddr,
        length: usize,
    ) -> Option<&'static [u8]> {
        Some(core::slice::from_raw_parts(
            addr as u64 as *const u8,
            length,
        ))
    }

    unsafe fn allocate(
        &mut self,
        length: usize,
    ) -> Option<(multiboot::information::PAddr, &mut [u8])> {
        None
    }

    unsafe fn deallocate(&mut self, addr: multiboot::information::PAddr) {
        unreachable!()
    }
}

#[no_mangle]
pub extern "C" fn kmain(boot_info_ptr: u64) -> ! {
    // ralloc::Allocator
    let mut bad_mman = BadMMAN;
    let bad_mman = unsafe { &mut *((&mut bad_mman) as *mut BadMMAN) as &'static mut BadMMAN };
    let ptr =
        unsafe { multiboot::information::Multiboot::from_ptr(boot_info_ptr, bad_mman).unwrap() };
    let boot_info = unsafe { &*((&ptr) as *const Multiboot) as &'static Multiboot };
    constants::check_const_correct();
    init::init(boot_info);
    {
        let mut executor = SimpleExecutor::new();
        executor.spawn(Task::new(shell::shell()));
        executor.run();
    }
    loop {}
}
//     init(boot_info);
//     constants::check_const_correct();
//     if constants::should_conio() {
//         println!("Should conio");
//     }
//     if constants::should_displayio() {
//         println!("Should displayio");
//     }
//     // TODO: this really really needs to work. Actually, we should use ld syms as per doug16k's suggestion.
//     // let kernel = unsafe {
//     //     alloc::slice::from_raw_parts(
//     //         boot_info.kernel_addr as *const u8,
//     //         boot_info.kernel_size as usize,
//     //     )
//     // };
//     // TODO: get unwind working
//     // crate::unwind::register_module(kernel);
//     let rsp: usize;
//     unsafe { asm!("mov rax, rsp", out("rax") rsp) };
//     println!("z: {:?}", VirtAddr::new(rsp as u64));
//     #[cfg(test)]
//     test_main();
//     #[cfg(not(test))]
//     {
//         let mut executor = SimpleExecutor::new();
//         executor.spawn(Task::new(shell::shell()));
//         executor.run();
//     }
//     if constants::is_test() {
//         exiting::exit_qemu(exiting::QemuExitCode::Success);
//     } else {
//         exiting::exit();
//     }
// }

#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> () {}
