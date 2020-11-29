use crate::{memory::map_to, prelude::*};
use x86_64::{
    structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
    VirtAddr,
};
use xmas_elf::{self, program::Type};

fn read_from_user<T>(ptr: *mut T) -> &'static T {
    if ptr as u64 >= 0xFFFF800000000000 {
        unsafe { ptr.as_ref().unwrap() }
    } else {
        panic!("Security violation: read attempted from {:p}", ptr);
    }
}
pub fn ensure_region_safe(ptr: *mut u8, len: usize) {
    if (ptr as u64) < 0xFFFF800000000000 {
        panic!("Security violation: read attempted from {:p}", ptr);
    } else if (ptr as u64).overflowing_add(len as u64).0 < 0xFFFF800000000000 {
        panic!("Security violation: read attempted from {:p}", ptr);
    }
}
fn user_gets(mut ptr: *mut u8, n: u64) -> String {
    let mut s = "".to_string();
    unsafe {
        for _ in 0..n {
            s += &((*read_from_user(ptr)) as char).to_string();
            ptr = ptr.offset(1);
        }
    }
    s
}

ezy_static! { SVC_MAP, BTreeMap<String, u64>, BTreeMap::new() }
fn freebox1() {
    match preempt::CURRENT_TASK.box1 {
        Some(s) => {
            // free it
            preempt::CURRENT_TASK.get().box1 = None;
            unsafe {
                Box::from_raw(s as *const [u8] as *mut [u8]);
            }
        }
        None => {}
    };
}
fn freebox2() {
    match preempt::CURRENT_TASK.box2 {
        Some(s) => {
            // free it
            preempt::CURRENT_TASK.get().box2 = None;
            unsafe {
                Box::from_raw(s as *const [u8] as *mut [u8]);
            }
        }
        None => {}
    };
}
pub fn syscall_handler(sysno: u64, arg1: u64, arg2: u64) -> u64 {
    match sysno {
        0 => {
            /* sys_exit */
            loop {
                preempt::yield_task();
            }
        }
        1 => {
            /* sys_bindbuffer */
            match preempt::CURRENT_TASK.box1 {
                Some(s) => {
                    // free it
                    preempt::CURRENT_TASK.get().box1 = None;
                    unsafe {
                        Box::from_raw(s as *const [u8] as *mut [u8]);
                    }
                }
                None => {}
            };
            let mut p = vec![];
            p.reserve_exact(arg2 as usize);
            unsafe {
                accelmemcpy(p.as_mut_ptr(), arg1 as *const u8, arg2 as usize);
            }
            preempt::CURRENT_TASK.get().box1 = Some(Box::leak(p.into_boxed_slice()));
            0
        }
        2 => {
            /* sys_getbufferlen */
            match preempt::CURRENT_TASK.box1 {
                Some(s) => s.len() as u64,
                None => 0,
            }
        }
        3 => {
            /* sys_readbuffer */
            match preempt::CURRENT_TASK.box1 {
                Some(s) => {
                    unsafe {
                        accelmemcpy(arg1 as *mut u8, s.as_ptr(), s.len());
                    };
                    s.len() as u64
                }
                None => 0,
            }
        }
        4 => {
            /* sys_swapbuffers */
            let buf1 = preempt::CURRENT_TASK.box1;
            let buf2 = preempt::CURRENT_TASK.box2;
            preempt::CURRENT_TASK.get().box2 = buf1;
            preempt::CURRENT_TASK.get().box1 = buf2;
            0
        }
        5 => {
            /* sys_send */
            let target = user_gets(arg1 as *mut u8, arg2);
            x86_64::instructions::interrupts::without_interrupts(|| {
                if ksvc::KSVC_TABLE.contains_key(&target) {
                    ksvc::KSVC_TABLE.get().get(&target).unwrap()();
                    return;
                }
                let p = *SVC_MAP.get().get(&target).unwrap();
                for r in preempt::TASK_QUEUE.get().iter_mut() {
                    if p == r.pid {
                        while !r.is_listening {
                            preempt::yield_task();
                        }
                        r.is_listening = false;
                        r.box1 = preempt::CURRENT_TASK.box1;
                        r.box2 = preempt::CURRENT_TASK.box2;
                        while !r.is_done {
                            preempt::yield_task();
                        }
                        freebox1();
                        freebox2();
                        preempt::CURRENT_TASK.get().box1 = r.box1;
                        preempt::CURRENT_TASK.get().box1 = r.box2;
                        r.box1 = None;
                        r.box2 = None;
                    }
                }
            });
            0
        }
        6 => {
            /* sys_listen */
            let name = user_gets(arg1 as *mut u8, arg2);
            preempt::CURRENT_TASK.get().is_listening = false;
            SVC_MAP.get().insert(name, preempt::CURRENT_TASK.pid);
            0
        }
        7 => {
            /* sys_accept */
            assert_eq!(
                *SVC_MAP
                    .get()
                    .get(&user_gets(arg1 as *mut u8, arg2))
                    .unwrap(),
                preempt::CURRENT_TASK.pid
            );
            preempt::CURRENT_TASK.get().is_done = false;
            preempt::CURRENT_TASK.get().is_listening = true;
            x86_64::instructions::interrupts::without_interrupts(|| {
                while preempt::CURRENT_TASK.get().is_listening {
                    preempt::yield_task()
                }
            });

            0
        }
        8 => {
            /* sys_exec */
            x86_64::instructions::interrupts::without_interrupts(|| {
                let l = preempt::CURRENT_TASK.box1;
                let r = preempt::CURRENT_TASK.box1;
                do_exec(l.unwrap(), r.unwrap());
                freebox1();
                freebox2();
                return;
            });
            0
        }
        9 => {
            /* sys_respond */
            x86_64::instructions::interrupts::without_interrupts(|| {
                preempt::CURRENT_TASK.get().is_done = true;
                preempt::yield_task();
                preempt::CURRENT_TASK.get().is_done = true;
            });
            0
        }
        10 => {
            /* sys_klog */
            print!("{}", user_gets(arg1 as *mut u8, arg2));
            0
        }
        11 => {
            /* sys_sbrk */
            let len = arg1;
            println!("[sbrk] BRK'ing {} bytes", len);
            let oldbrk = preempt::CURRENT_TASK.program_break;
            let newbrk = ((oldbrk + len + 4095) / 4096) * 4096;
            println!("[sbrk] {:#x?} => {:#x?}", oldbrk, newbrk);
            preempt::CURRENT_TASK.get().program_break = newbrk;
            for i in 0..(((newbrk - oldbrk) / 4096) + 1) {
                let pageaddr = oldbrk + i * 4096;

                let data = crate::memory::mpage();
                // pages.push(data);
                if pageaddr < 0xFFFF800000000000 {
                    panic!("Invalid target for sbrk!");
                }
                map_to(
                    VirtAddr::from_ptr(data),
                    VirtAddr::new(pageaddr),
                    PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE,
                );
            }
            dbg!(preempt::CURRENT_TASK.get().program_break);
            return newbrk;
        }
        _ => (-1 as i64) as u64,
    }
}
#[no_mangle]
unsafe extern "C" fn syscall_trampoline_rust(sysno: u64, arg1: u64, arg2: u64) -> u64 {
    syscall_handler(sysno, arg1, arg2)
}
extern "C" {
    static mut RSP_PTR: u64;
}
pub fn get_rsp_ptr() -> VirtAddr {
    unsafe { VirtAddr::new(RSP_PTR) }
}
pub fn set_rsp_ptr(va: VirtAddr) {
    unsafe {
        RSP_PTR = va.as_u64();
    }
}
pub fn alloc_rsp_ptr() -> VirtAddr {
    const STACK_SIZE: usize = 4096 * 5;
    let stack_start = VirtAddr::from_ptr(crate::memory::malloc(STACK_SIZE));
    let stack_end = stack_start + STACK_SIZE;
    stack_end
}
pub fn init_rsp_ptr() {
    set_rsp_ptr(alloc_rsp_ptr());
}
#[naked]
pub unsafe fn new_syscall_trampoline() {
    asm!(
        "
        cli
        push rcx
        push r11
        push rbp
        mov rbp, rsp
        mov rsp, [RSP_PTR]
        push rbp
        push rbx
        push rcx
        push rdx
        push r12
        push r13
        push r14
        push r15
        mov rbp, rsp
        call syscall_trampoline_rust
        mov rsp, rbp
        pop r15
        pop r14
        pop r13
        pop r12
        pop rdx
        pop rcx
        pop rbx
        pop rbp
        mov rsp, rbp
        pop rbp
        pop r11
        pop rcx
    just_a_brk:
        sysretq
    .global RSP_PTR
    RSP_PTR:
        .space 0x8, 0x00
    "
    );
}
unsafe fn accelmemcpy(to: *mut u8, from: *const u8, size: usize) {
    if size < 8 {
        faster_rlibc::memcpy(to, from, size);
        return;
    }
    if size & 0x07 != 0 {
        faster_rlibc::memcpy(
            to.offset((size & 0xfffffffffffffff8) as isize),
            from.offset((size & 0xfffffffffffffff8) as isize),
            size & 0x07,
        );
    }
    faster_rlibc::fastermemcpy(to, from, size & 0xfffffffffffffff8);
}
ezy_static! { PID_COUNTER, u64, 1 }
pub fn mkpid(ppid: u64) -> u64 {
    let r = (*PID_COUNTER.get() << 1) | (ppid & 1);
    *PID_COUNTER.get() += 1;
    r
}
pub fn getpid() -> u64 {
    preempt::CURRENT_TASK.pid
}
pub fn loaduser() {
    init_rsp_ptr();
    let slice = include_bytes!("../build/test.elf");
    let mut pages: Vec<*mut u8> = vec![];
    let exe = xmas_elf::ElfFile::new(slice).unwrap();
    let mut program_break: u64 = 0xFFFF800000000000;
    for ph in exe.program_iter() {
        if ph.get_type().unwrap() == Type::Load {
            let mut flags = PageTableFlags::NO_EXECUTE | PageTableFlags::PRESENT;
            // if ph.flags().is_read() {
            flags |= PageTableFlags::USER_ACCESSIBLE;
            // }
            // if ph.flags().is_execute() {
            flags ^= PageTableFlags::NO_EXECUTE;
            // }
            // if ph.flags().is_write() {
            flags |= PageTableFlags::WRITABLE;
            // }
            println!("Flags: {:?} addr: {:#x?}", flags, ph.virtual_addr());
            let page_count = (ph.file_size() + 4095 + (ph.virtual_addr() % 4096)) / 4096;
            for i in 0..page_count {
                let data = crate::memory::mpage();
                pages.push(data);
                if ph.virtual_addr() + (i * 4096) < 0xFFFF800000000000 {
                    panic!("Invalid target for ELF loader!");
                }
                map_to(
                    VirtAddr::from_ptr(data),
                    VirtAddr::new(ph.virtual_addr() + (i * 4096)),
                    flags,
                );
            }
            let maybe_new_program_break = ph.virtual_addr() + (page_count * 4096);
            program_break = if maybe_new_program_break < program_break {
                program_break
            } else {
                maybe_new_program_break
            };

            unsafe {
                accelmemcpy(
                    ph.virtual_addr() as *mut u8,
                    slice.as_ptr().offset(ph.offset() as isize),
                    ph.file_size() as usize,
                );
            }
        }
    }
    // now initialize all the necessary fields.

    // To free just fpage() all of the `pages`
    preempt::CURRENT_TASK.get().pid = mkpid(preempt::CURRENT_TASK.pid);
    preempt::CURRENT_TASK.get().program_break = ((program_break + 4095) / 4096) * 4096;
    unsafe {
        jump_user(exe.header.pt2.entry_point());
    }
}

pub fn do_exec(kernel: &[u8], argvblob: &[u8]) {
    // let's test multitasking
    let slice = kernel;
    let mut pages: Vec<*mut u8> = vec![];
    let exe = xmas_elf::ElfFile::new(slice).unwrap();
    let mut program_break: u64 = 0xFFFF800000000000;
    for ph in exe.program_iter() {
        let mut flags = PageTableFlags::NO_EXECUTE | PageTableFlags::PRESENT;
        if ph.flags().is_read() {
            flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        if ph.flags().is_execute() {
            flags ^= PageTableFlags::NO_EXECUTE;
        }
        if ph.flags().is_write() {
            flags |= PageTableFlags::WRITABLE;
        }
        println!("Flags: {:?} addr: {:#x?}", flags, ph.virtual_addr());
        let page_count = (ph.file_size() + 4095) / 4096;
        for i in 0..page_count {
            let data = crate::memory::mpage();
            pages.push(data);
            if ph.virtual_addr() + (i * 4096) < 0xFFFF800000000000 {
                panic!("Invalid target for ELF loader!");
            }
            map_to(
                VirtAddr::from_ptr(data),
                VirtAddr::new(ph.virtual_addr() + (i * 4096)),
                flags,
            );
        }
        let maybe_new_program_break = ph.virtual_addr() + (page_count * 4096);
        program_break = if maybe_new_program_break < program_break {
            program_break
        } else {
            maybe_new_program_break
        };
        unsafe {
            accelmemcpy(
                ph.virtual_addr() as *mut u8,
                slice.as_ptr().offset(ph.offset() as isize),
                ph.file_size() as usize,
            );
        }
    }
    // now initialize all the necessary fields.

    // To free just fpage() all of the `pages`
    preempt::CURRENT_TASK.get().pid = mkpid(preempt::CURRENT_TASK.pid);
    preempt::CURRENT_TASK.get().program_break = program_break;
    preempt::task_alloc(|| unsafe {
        let ve: Vec<u8> = argvblob.iter().map(|f| *f).collect();
        preempt::CURRENT_TASK.get().box1 = Some(ve.leak());
        jump_user(exe.header.pt2.entry_point());
    });
}
unsafe fn jump_user(addr: u64) {
    asm!("
    mov ds,ax
    mov es,ax 
    mov fs,ax 
    mov gs,ax

    mov rsi, rsp
    push rax
    push rsi
    push 0x200
    push rdx
    push rdi
    iretq", in("rdi") addr, in("ax") 0x1b, in("dx") 0x23, in("rsi") 0);
    unreachable!();
}
