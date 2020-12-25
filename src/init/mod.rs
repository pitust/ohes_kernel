use core::ffi::c_void;

use x86_64::{
    instructions::{
        segmentation::set_cs,
        segmentation::{load_ds, load_fs, load_gs, load_ss},
        tables::load_tss,
    },
    registers::{
        model_specific::Star,
        model_specific::{Efer, EferFlags, LStar, SFMask},
        rflags::RFlags,
    },
};

use crate::prelude::*;
// handle init

fn run_task<T: Fn()>(t: &str, f: T) {
    println!("[kinit] task {}", t);
    f();
}
fn gdt() {
    interrupts::GDT.0.load();
    unsafe {
        set_cs(interrupts::GDT.1.code_selector);
        load_ds(interrupts::GDT.1.data_selector);
        load_fs(interrupts::GDT.1.data_selector);
        load_gs(interrupts::GDT.1.data_selector);
        load_ss(interrupts::GDT.1.data_selector);
        load_tss(interrupts::GDT.1.tss_selector);
    }
}
pub fn init(boot_info: &'static multiboot::information::Multiboot) {
    println!("[kinit] Setting up Oh Es");
    println!("[kinit] [mman] initializing...");
    unsafe {
        *memory::FRAME_ALLOC.get() = Some(memory::BootInfoFrameAllocator::init(
            boot_info.upper_memory_bound().unwrap(),
        ));
    }
    println!("[kinit] [mman] we have frame allocation!");

    memory::allocator::init_heap().expect("Heap init failed");
    println!("[kinit] [mman] heap ready.");
    run_task("idt", || {
        interrupts::init_idt();
    });
    run_task("pit", || {
        interrupts::init_timer(1);
    });
    run_task("gdt", gdt);
    let b = box 3;
    println!("{}", b);
    run_task("io.general", || {
        io::proper_init_for_iodevs(boot_info);
    });
    run_task("status", || {
        println!("We have a liftoff! Internal kernel mman done.");
        let c = x86::cpuid::CpuId::new();
        println!(
            "Booting Oh Es on {}",
            c.get_vendor_info().unwrap().as_string()
        );
        let r1 = x86::cpuid::cpuid!(0x80000002);
        let r2 = x86::cpuid::cpuid!(0x80000003);
        let r3 = x86::cpuid::cpuid!(0x80000004);
        let mut bytes = vec![];
        bytes.push(r1.eax);
        bytes.push(r1.ebx);
        bytes.push(r1.ecx);
        bytes.push(r1.edx);
        bytes.push(r2.eax);
        bytes.push(r2.ebx);
        bytes.push(r2.ecx);
        bytes.push(r2.edx);
        bytes.push(r3.eax);
        bytes.push(r3.ebx);
        bytes.push(r3.ecx);
        bytes.push(r3.edx);
        let d = unsafe { core::slice::from_raw_parts(bytes.as_mut_ptr() as *mut u8, 3 * 4 * 4) };
        println!(" + Brand is {}", String::from_utf8(d.to_vec()).unwrap());

        match c.get_hypervisor_info() {
            Some(hi) => {
                println!(" + We are on {:?}", hi.identify());
            }
            None => {}
        };
    });
    run_task("task_queue.init", || {
        preempt::TASK_QUEUE.get();
    });

    // Set up syscalls
    run_task("regs", || {
        run_task("regs.efer", || unsafe {
            Efer::update(|a| {
                *a |= EferFlags::SYSTEM_CALL_EXTENSIONS | EferFlags::NO_EXECUTE_ENABLE;
            });
        });
        run_task("regs.lstar", || {
            LStar::write(VirtAddr::from_ptr(
                crate::userland::new_syscall_trampoline as *const u8,
            ));
        });
        run_task("regs.sfmask", || {
            SFMask::write(RFlags::INTERRUPT_FLAG);
        });
        run_task("regs.star", || {
            Star::write(
                interrupts::GDT.1.usercode,
                interrupts::GDT.1.userdata,
                interrupts::GDT.1.code_selector,
                interrupts::GDT.1.data_selector,
            )
            .unwrap();
        });
    });
    run_task("io.device.kbdint", || {
        task::keyboard::KEY_QUEUE.init_once(|| crossbeam_queue::ArrayQueue::new(100));
    });

    run_task("unwind", || {

        unsafe {
            unwind::__register_frame((&memory::es) as *const u8 as *mut u8 as *mut c_void, (&memory::esz) as *const u8 as u64 as usize);
        }
    });
    run_task("ksvc", || {
        ksvc::ksvc_init();
    });
    run_task("enable_int", || {
        x86_64::instructions::interrupts::enable();
    });
}
