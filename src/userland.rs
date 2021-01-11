use core::usize;

use crate::{
    drive::{gpt::GetGPTPartitions, RODev},
    memory::map_to,
    prelude::*,
};
use kmacros::handle_read;
use preempt::WakeType;
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
    let mut s = vec![];
    unsafe {
        for _ in 0..n {
            s.push(*read_from_user(ptr));
            ptr = ptr.offset(1);
        }
    }
    String::from_utf8(s).unwrap()
}
pub struct Service {
    pid: u64,
    is_active: bool,
    activate_pids: LinkedList<u64>,
}

ezy_static! { SVC_MAP, spin::Mutex<BTreeMap<String, Service>>, spin::Mutex::new(BTreeMap::new()) }
fn freebox1() {
    match task().box1 {
        Some(s) => {
            // free it
            task().box1 = None;
            unsafe {
                Box::from_raw(s as *const [u8] as *mut [u8]);
            }
        }
        None => {}
    };
}
fn freebox2() {
    match task().box2 {
        Some(s) => {
            // free it
            task().box2 = None;
            unsafe {
                Box::from_raw(s as *const [u8] as *mut [u8]);
            }
        }
        None => {}
    };
}
pub fn syscall_handler(sysno: u64, arg1: u64, arg2: u64) -> u64 {
    dprintln!(" ===> enter {} {:#x?}", task().pid, sysno);
    let v = match sysno {
        0 => {
            /* sys_exit */
            loop {
                preempt::yield_task();
            }
        }
        1 => {
            /* sys_bindbuffer */
            match task().box1 {
                Some(s) => {
                    // free it
                    task().box1 = None;
                    unsafe {
                        Box::from_raw(s as *const [u8] as *mut [u8]);
                    }
                }
                None => {}
            };
            let mut p = vec![];
            p.resize(arg2 as usize, 0);
            unsafe {
                accelmemcpy(p.as_mut_ptr(), arg1 as *const u8, arg2 as usize);
            }
            task().box1 = Some(Box::leak(p.into_boxed_slice()));
            0
        }
        2 => {
            /* sys_getbufferlen */
            match task().box1 {
                Some(s) => s.len() as u64,
                None => 0,
            }
        }
        3 => {
            /* sys_readbuffer */
            match task().box1 {
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
            let buf1 = task().box1;
            let buf2 = task().box2;
            task().box2 = buf1;
            task().box1 = buf2;
            0
        }
        5 => {
            /* sys_send */
            let target = user_gets(arg1 as *mut u8, arg2);
            if target == "kfs" {
                ksvc::dofs();
                dprintln!(" <=== exit {}", task().pid);
                return 0;
            }
            x86_64::instructions::interrupts::without_interrupts(|| {
                if ksvc::KSVC_TABLE.contains_key(&target) {
                    ksvc::KSVC_TABLE.get().get(&target).unwrap()();
                    dprintln!(" <=== exit {}", task().pid);
                    return;
                }
                let mut svclock = SVC_MAP.lock();
                let p = svclock.get_mut(&target).unwrap();
                for r in preempt::TASK_QUEUE.get().iter_mut() {
                    if p.pid == r.pid {
                        while !p.is_active {
                            p.activate_pids.push_back(task().pid);
                            task().needs_wake = true;
                            preempt::yield_task();
                            assert_eq!(
                                task().wakeop,
                                Some(preempt::Wakeop {
                                    wake_type: preempt::WakeType::WakeServerReady,
                                    waker: p.pid
                                })
                            );
                        }
                        p.activate_pids.push_back(task().pid);
                        task().needs_wake = true;
                        preempt::yield_task();
                        assert_eq!(
                            task().wakeop,
                            Some(preempt::Wakeop {
                                wake_type: preempt::WakeType::WakeConnection,
                                waker: p.pid
                            })
                        );
                        r.box1 = task().box1;
                        r.box2 = task().box2;
                        task().needs_wake = true;
                        preempt::yield_task();
                        assert_eq!(
                            task().wakeop,
                            Some(preempt::Wakeop {
                                wake_type: preempt::WakeType::WakeResponded,
                                waker: p.pid
                            })
                        );
                        freebox1();
                        freebox2();
                        task().box1 = r.box1;
                        task().box1 = r.box2;
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
            SVC_MAP.lock().insert(
                name,
                Service {
                    pid: task().pid,
                    activate_pids: LinkedList::new(),
                    is_active: false,
                },
            );
            0
        }
        7 => {
            /* sys_accept */
            let nejm = user_gets(arg1 as *mut u8, arg2);
            let mut ent = SVC_MAP.lock();
            let svc = ent.get_mut(&nejm).unwrap();
            let q = svc.activate_pids.pop_front();
            if q != None {
                let q = q.unwrap();
                for r in preempt::TASK_QUEUE.get().iter_mut() {
                    if q == r.pid {
                        assert!(r.needs_wake);
                        r.wakeop = Some(preempt::Wakeop {
                            wake_type: preempt::WakeType::WakeServerReady,
                            waker: task().pid,
                        });
                        r.needs_wake = false;
                        break;
                    }
                }
            }
            task().needs_wake = true;
            preempt::yield_task();
            assert_eq!(task().wakeop.unwrap().wake_type, WakeType::WakeConnection);

            task().currently_responding_to = task().wakeop.unwrap().waker;
            0
        }
        8 => {
            /* sys_exec */
            x86_64::instructions::interrupts::without_interrupts(|| {
                let l = task().box1;
                do_exec(l.unwrap());
            });
            0
        }
        9 => {
            /* sys_respond */
            x86_64::instructions::interrupts::without_interrupts(|| {
                let resp = task().currently_responding_to;

                for r in preempt::TASK_QUEUE.get().iter_mut() {
                    if resp == r.pid {
                        assert!(r.needs_wake);
                        r.needs_wake = false;
                        r.wakeop = Some(preempt::Wakeop {
                            wake_type: preempt::WakeType::WakeResponded,
                            waker: task().pid,
                        });
                        sched_yield();
                        assert_eq!(task().box1, None);
                        assert_eq!(task().box2, None);
                    }
                }
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
            let oldbrk = task().program_break;
            let newbrk = ((oldbrk + len + 4095) / 4096) * 4096;
            task().program_break = newbrk;
            for i in 0..(((newbrk - oldbrk) / 4096) + 1) {
                let pageaddr = oldbrk + i * 4096;

                let data = crate::memory::mpage();
                // pages.push(data);
                if pageaddr < 0xFFFF800000000000 {
                    panic!("Invalid target for sbrk! {:#x?}", pageaddr);
                }
                map_to(
                    VirtAddr::from_ptr(data),
                    VirtAddr::new(pageaddr),
                    PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE,
                );
                preempt::yield_task();
            }
            dprintln!(" <=== exit {}", task().pid);
            return newbrk;
        }
        _ => (-1 as i64) as u64,
    };

    dprintln!(" <=== exit {}", task().pid);
    v
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
pub fn alloc_rsp_ptr(stack_name: String) -> VirtAddr {
    const STACK_SIZE: usize = 4096 * 5;
    let stack_start = VirtAddr::from_ptr(crate::memory::malloc(STACK_SIZE));
    let stack_end = stack_start + STACK_SIZE;
    stack_canaries::add_canary(stack_start, stack_name, STACK_SIZE as u64);
    stack_end
}
pub fn init_rsp_ptr(stack_name: String) {
    set_rsp_ptr(alloc_rsp_ptr(stack_name));
}
#[naked]
pub unsafe extern "C" fn new_syscall_trampoline() {
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
    ",
        options(noreturn)
    );
}
unsafe fn accelmemcpy(to: *mut u8, from: *const u8, size: usize) {
    x86_64::instructions::interrupts::without_interrupts(|| {
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
    });
}

counter!(PID_COUNTER);
pub fn mkpid() -> u64 {
    PID_COUNTER::inc() as u64
}
pub fn getpid() -> u64 {
    task().pid
}
#[handle_read]
pub fn readfs(path: &str) -> &[u8] {
    panic!("asds");
}

pub fn loaduser() {
    init_rsp_ptr("syscall-stack:/bin/init".to_string());
    let loaded_init = readfs("/bin/init");
    let mut pages: Vec<*mut u8> = vec![];
    let exe = xmas_elf::ElfFile::new(&loaded_init).unwrap();
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
                    loaded_init.as_ptr().offset(ph.offset() as isize),
                    ph.file_size() as usize,
                );
            }
        }
    }
    // now initialize all the necessary fields.

    // To free just fpage() all of the `pages`
    task().pid = mkpid();
    task().program_break = ((program_break + 4095) / 4096) * 4096;
    unsafe {
        jump_user(exe.header.pt2.entry_point());
    }
}

pub fn do_exec(kernel: &[u8]) {
    let slice = kernel.to_vec();
    let ve = task().box2.take();
    freebox1();
    freebox2();
    let path = String::from_utf8(slice).unwrap();
    let path2 = path.clone();
    preempt::task_alloc(
        move || unsafe {
            x86_64::instructions::interrupts::disable();
            let slice = readfs(&path);

            let ncr3 = main::forkp();
            let mut pages: Vec<*mut u8> = vec![];
            let exe = xmas_elf::ElfFile::new(&slice).unwrap();
            let mut program_break: u64 = 0xFFFF800000000000;
            for ph in exe.program_iter() {
                let mut flags = PageTableFlags::NO_EXECUTE | PageTableFlags::PRESENT;
                flags |= PageTableFlags::USER_ACCESSIBLE;
                flags ^= PageTableFlags::NO_EXECUTE;
                flags |= PageTableFlags::WRITABLE;
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
                accelmemcpy(
                    ph.virtual_addr() as *mut u8,
                    slice.as_ptr().offset(ph.offset() as isize),
                    ph.file_size() as usize,
                );
            }
            x86_64::registers::control::Cr3::write(ncr3.0, ncr3.1);
            task().box1 = ve;
            task().pid = mkpid();
            task().program_break = program_break;
            x86_64::instructions::interrupts::enable();
            jump_user(exe.header.pt2.entry_point());
        },
        format!("syscall-stack:{}", path2.clone()),
    );
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
