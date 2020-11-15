#![no_std]
#![feature(global_asm)]
global_asm!(include_str!("main.s"));

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Jmpbuf {
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rsi: u64,
}
impl Jmpbuf {
    pub fn new() -> Jmpbuf {
        Jmpbuf {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: 0,
            rip: 0,
            rsi: 0,
        }
    }
}
pub type PushJmpbuf = extern "C" fn(&mut Jmpbuf);
extern {
    pub fn setjmp(buf: &mut Jmpbuf) -> i32;
    pub fn longjmp(buf: &Jmpbuf, val: i32) -> !;
    pub fn changecontext(fcn: PushJmpbuf, ref_to_a_buffer: &mut Jmpbuf);
}