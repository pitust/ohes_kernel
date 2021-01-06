// preemtive multitasking
use crate::prelude::*;
use safety_here::Jmpbuf;
use x86_64::{
    instructions::tables::{lgdt, load_tss},
    registers::control::Cr3,
    structures::{
        gdt::GlobalDescriptorTable, paging::mapper::MapToError, paging::FrameAllocator,
        paging::Mapper, paging::Page, paging::PageTableFlags, paging::PhysFrame, paging::Size4KiB,
        tss::TaskStateSegment,
    },
    VirtAddr,
};
static STACKS: AtomicU64 = AtomicU64::new(0x18000u64);
pub fn stack_alloc(stack_size: u64) -> Result<*const u8, MapToError<Size4KiB>> {
    let mut frame_alloc = crate::memory::FRAME_ALLOC
        .get()
        .expect("A frame allocator was not made yet");
    // frame_alloc.show();
    let base = STACKS.fetch_add(stack_size, Ordering::Relaxed);
    let page_range = {
        let stack_start = VirtAddr::new(base as u64);
        let stack_end = stack_start + stack_size - 1u64;
        let heap_start_page = Page::containing_address(stack_start);
        let heap_end_page = Page::containing_address(stack_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        println!("MAP: {:#?} -> {:#?}", page, frame);
        unsafe {
            memory::get_mapper()
                .map_to(page, frame, flags, &mut frame_alloc)?
                .flush();
        };
    }
    Ok(base as *const u8)
}
static TASK_QUEUE_CUR: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug)]
enum WakeType {
    WakeConnection,
    WakeServerReady,
    WakeProcessExited { code: u64 },
    WakeResponded,
}

#[derive(Copy, Clone, Debug)]
pub struct Wakeop {
    wake_type: WakeType,
    waker: u64,
    is_cross_task_call: bool,
}
// privesc would be CURRENT_TASK.get().uid = 0. just sayin' you know.
#[derive(Copy, Clone, Debug)]
pub struct Task {
    pub state: Jmpbuf,
    pub rsp0: VirtAddr,
    pub rsp_ptr: VirtAddr,
    pub pid: u64,
    pub box1: Option<&'static [u8]>,
    pub box2: Option<&'static [u8]>,
    pub program_break: u64,
    pub wakeop: Option<Wakeop>,
    pub needs_wake: bool,
    pub uid: i32,
}
ezy_static! { TASK_QUEUE, Vec<Task>, vec![Task { state: Jmpbuf::new(), rsp0: crate::interrupts::get_rsp0(), rsp_ptr: crate::userland::alloc_rsp_ptr("syscall-stack:/bin/init".to_string()), pid: 1, box1: None, box2: None, program_break: 0, wakeop: None, needs_wake: false, uid: -1 }] }
ezy_static! { CURRENT_TASK, Task, Task { state: Jmpbuf::new(), rsp0: crate::interrupts::get_rsp0(), rsp_ptr: crate::userland::alloc_rsp_ptr("fake stack".to_string()), pid: 1, box1: None, box2: None, program_break: 0, wakeop: None, needs_wake: false, uid: -1 } }
extern "C" fn get_next(buf: &mut Jmpbuf) {
    let tq = TASK_QUEUE.get();
    let mut ct = CURRENT_TASK.clone();
    ct.state = buf.clone();
    tq[TASK_QUEUE_CUR.fetch_add(1, Ordering::Relaxed)] = ct;
    TASK_QUEUE_CUR.store(
        TASK_QUEUE_CUR.load(Ordering::Relaxed) % tq.len(),
        Ordering::Relaxed,
    );
    let q = tq[TASK_QUEUE_CUR.load(Ordering::Relaxed) % tq.len()];
    *CURRENT_TASK.get() = q;
    crate::interrupts::set_rsp0(q.rsp0);
    crate::userland::set_rsp_ptr(q.rsp_ptr);
    *buf = q.state;
}
pub fn yield_task() -> () {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let c = Cr3::read();
        // set ourselves up as a task.
        let mut bufl = Jmpbuf::new();

        unsafe {
            safety_here::changecontext(get_next, &mut bufl);
        }
        unsafe {
            Cr3::write(c.0, c.1);
        }
    });
}
// we allow this here; this is a setup call for a new task
#[allow(improper_ctypes_definitions)]
extern "C" fn setup_call(
    _1: u64,
    _2: u64,
    _3: u64,
    _4: u64,
    _5: u64,
    _6: u64,
    fcn: fn(arg: u64) -> (),
    arg: u64,
) -> ! {
    x86_64::instructions::interrupts::enable();
    fcn(arg);
    loop {
        yield_task();
    }
}

pub fn task_alloc<T: FnOnce<()>>(f: T, stknm: String) {
    fn run_task_ll<T: FnOnce<()>>(arg: u64) {
        let b = unsafe { Box::from_raw(arg as *mut T) };
        b();
    }
    let b = Box::new(f);
    let ptr = Box::leak(b) as *const T;
    jump_to_task(run_task_ll::<T>, ptr as u64, stknm);
}
fn jump_to_task(newfcn: fn(arg: u64) -> (), arg: u64, stknm: String) {
    const STACK_SIZE_IN_QWORDS: usize = 1024;
    let end_of_stack = STACK_SIZE_IN_QWORDS - 2;
    let mut stack: Box<[u64]> = box [0; STACK_SIZE_IN_QWORDS];
    let index: usize = end_of_stack - 1; // Represents the callee saved registers
    stack[end_of_stack] = newfcn as u64;
    stack[end_of_stack + 1] = arg;
    let stack_ptr = Box::into_raw(stack);
    let stack_ptr_as_usize = stack_ptr as *mut u64 as usize;
    let stack_ptr_start = stack_ptr_as_usize + (index * core::mem::size_of::<usize>());

    let mut b = safety_here::Jmpbuf::new();
    b.rbx = 0;
    b.rbp = 0;
    b.r12 = 0;
    b.r13 = 0;
    b.r14 = 0;
    b.r15 = 0;
    b.rsp = stack_ptr_start as u64;
    b.rip = setup_call as *const u8 as u64;
    b.rsi = newfcn as *const u8 as u64;
    x86_64::instructions::interrupts::without_interrupts(|| {
        TASK_QUEUE.get().push(Task {
            state: b,
            rsp0: crate::interrupts::alloc_rsp0(),
            rsp_ptr: crate::userland::alloc_rsp_ptr(stknm),
            pid: 1,
            box1: None,
            box2: None,
            program_break: 0,
            wakeop: None,
            needs_wake: false,
            uid: -1,
        });
    });
}

#[cfg(test)]
static VAL: AtomicU64 = AtomicU64::new(3);
