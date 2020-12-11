use crate::{print, println};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use x86_64::{
    instructions::port::{Port, PortRead, PortWrite},
    structures::gdt::SegmentSelector,
};
use x86_64::{
    registers::control::Cr2,
    structures::paging::{Mapper, Page, PhysFrame, Size4KiB},
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_STACK_INDEX: u16 = 1;
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: crate::shittymutex::Mutex<ChainedPics> =
    crate::shittymutex::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // 0
    Keyboard,             // 1
    _Cascade,             // 2
    COM2,                 // 3
    COM1,                 // 4
    LPT2,                 // 5
    Floppy,               // 6
    UnreliableLPT1,       // 7
    CMOS,                 // 8
    Free1,                // 9
    Free2,
    Free3,
    Mouse,
    FPU,
    PrimaryATA,
    SecondaryATA,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK_DATA: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK_DATA });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.interrupt_stack_table[PAGE_FAULT_STACK_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 128;
            static mut STACK_DATA: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK_DATA });
            let stack_end = stack_start + STACK_SIZE;
            stack_end + (4096 as usize)
        };
        tss.privilege_stack_table[0] = {
            // stack when going back to kmode.
            // for real only used
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK_DATA: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK_DATA });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };
}
pub fn get_rsp0() -> VirtAddr {
    TSS.privilege_stack_table[0]
}
pub fn set_rsp0(va: VirtAddr) {
    let tss_borrow: &TaskStateSegment = &TSS;
    let tss_mut = unsafe { &mut *(tss_borrow as *const TaskStateSegment as *mut TaskStateSegment) };
    tss_mut.privilege_stack_table[0] = va;
}
pub fn alloc_rsp0() -> VirtAddr {
    const STACK_SIZE: usize = 4096 * 5;
    let stack_start = VirtAddr::from_ptr(crate::memory::malloc(STACK_SIZE));
    let stack_end = stack_start + STACK_SIZE;
    stack_end
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment()); // 0x08
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment()); // 0x10
        let userdata = gdt.add_entry(Descriptor::user_data_segment()); // 0x18
        let usercode = gdt.add_entry(Descriptor::user_code_segment()); // 0x20
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS)); // 0x28
        (gdt, Selectors { code_selector, tss_selector, data_selector, usercode, userdata })
    };
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(PAGE_FAULT_STACK_INDEX);
        }
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.general_protection_fault.set_handler_fn(gpe);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt[InterruptIndex::COM1.as_usize()].set_handler_fn(com1_handler);
        idt[InterruptIndex::COM2.as_usize()].set_handler_fn(com1_handler);
        idt[InterruptIndex::PrimaryATA.as_usize()].set_handler_fn(com1_handler);
        unsafe { PICS.get().initialize() };
        idt
    };
}

lazy_static! {
    static ref KEYBOARD: crate::shittymutex::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        crate::shittymutex::Mutex::new(Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::Ignore
        ));
}

#[derive(Debug, Copy, Clone)]
pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
    pub usercode: SegmentSelector,
    pub userdata: SegmentSelector,
}
extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame,);
}
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "Double fault at: \n{:#?}\nError code: {}",
        stack_frame, error_code
    );
}
extern "x86-interrupt" fn gpe(stack_frame: &mut InterruptStackFrame, error_code: u64) -> () {
    panic!("#GP at: \n{:#?}\nError code: {}", stack_frame, error_code);
}
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut InterruptStackFrame) -> () {
    loop {}
    panic!("Invalid opcode at: \n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("ifF: {}", x86_64::instructions::interrupts::are_enabled());
    let addr = Cr2::read();
    let f = crate::memory::get_flags_for(addr);
    println!("F: {:?}", f);
    // IF it was a COW page AND it was a write, copy and make writable (aka Copy On Write)
    // if error_code.intersects(PageFaultErrorCode::CAUSED_BY_WRITE) {
    //     let addr = Cr2::read();
    //     // reverse lookup to page frame
    //     let f = crate::memory::get_flags_for(addr);
    //     match f {
    //         Some(mut flags) => {
    //             if flags.intersects(crate::memory::allocator::COW_PAGE) {
    //                 // Yes indeed
    //                 // We need a new page
    //                 let pgn = crate::memory::mpage();
    //                 // Copy the data
    //                 unsafe { faster_rlibc::fastermemcpy(pgn, addr.as_ptr(), 4096); }
    //                 crate::memory::MAPPER.get().unmap(Page::<Size4KiB>::containing_address(addr));
    //                 flags.remove(crate::memory::allocator::COW_PAGE);
    //                 crate::memory::map_to(addr, VirtAddr::from_ptr(pgn), flags);
    //                 // Go back
    //                 return;
    //             }
    //         }
    //         None => {}
    //     }
    // }
    // loop
    crate::io::Printer.set_color(255, 0, 0);
    println!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nWhy?: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
    if (error_code & PageFaultErrorCode::MALFORMED_TABLE) == PageFaultErrorCode::MALFORMED_TABLE {
        panic!("A malformed table was detected.");
    }
    if (error_code & PageFaultErrorCode::INSTRUCTION_FETCH) == PageFaultErrorCode::INSTRUCTION_FETCH
    {
        panic!(
            "Error: We tried to run code at an invalid address {:?}",
            Cr2::read()
        );
    }
    if (error_code & PageFaultErrorCode::CAUSED_BY_WRITE) == PageFaultErrorCode::CAUSED_BY_WRITE {
        panic!(
            "Error: We tried to write to an invalid address {:?} from {:?}",
            Cr2::read(),
            stack_frame.instruction_pointer
        );
    }
    // panic!("Page fault");
    loop {}
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        PICS.get()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    if !crate::constants::is_test() {
        crate::preempt::yield_task();
    }
    // println!("if: {}", x86_64::instructions::interrupts::are_enabled());
}
extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::disable();
    let keyboard = KEYBOARD.get();
    let scancode: u8 = unsafe { u8::read_from_port(0x60) };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(dk) = keyboard.process_keyevent(key_event) {
            crate::task::keyboard::key_enque(dk);
        }
    }
    unsafe {
        PICS.get()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
    drop(keyboard);
    x86_64::instructions::interrupts::enable();
}
extern "x86-interrupt" fn com1_handler(_stack_frame: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::disable();
    unsafe {
        PICS.get()
            .notify_end_of_interrupt(InterruptIndex::COM1.as_u8());
    }
    x86_64::instructions::interrupts::enable();
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_timer(freq: u32) {
    unsafe {
        u8::write_to_port(0x43, 0x34);
        u8::write_to_port(0x40, ((1193182 / freq) & 0xff) as u8);
        u8::write_to_port(0x40, ((1193182 / freq) >> 8) as u8);
    }
}

const PIC1_DATA: u16 = 0x21;
const PIC2_DATA: u16 = 0xa1;
pub fn noirq(mut line: u8) {
    let port: u16;
    let value: u8;

    if line < 8 {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        line -= 8;
    }
    unsafe {
        value = u8::read_from_port(port) | (1 << line);
        u8::write_to_port(port, value);
    }
}
pub fn goirq(mut line: u8) {
    let port: u16;
    let value: u8;

    if line < 8 {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        line -= 8;
    }
    unsafe {
        value = u8::read_from_port(port) & (0xff ^ (1 << line));
        u8::write_to_port(port, value);
    }
}
