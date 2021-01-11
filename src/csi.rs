use core::panic::PanicInfo;

use crate::prelude::*;

pub fn csi(panicinfo: &PanicInfo) {
    println!("~~ Crash Site Investigation Report ~~");
    println!(" => Message: {}", panicinfo.message().unwrap());
    println!(" => Crash at: {}", panicinfo.location().unwrap());
    println!(" => We are pid: {}", task().pid);
    println!(" ======= RSP Pointers =======");
	for p in stack_canaries::CANARIES.get() {
		println!("  (addr = {:?} | name = {} | len = {:#x?})", p.0, p.1, p.2);
	}
    println!(" ======= Processes =======");
    for tsk in preempt::TASK_QUEUE.get() {
		println!(" == bgnps ==");
        println!(" ==> pid {}", tsk.pid);
        for p in stack_canaries::CANARIES.get() {
            if p.0 == tsk.rsp_ptr {
                println!(" ==> rsp_ptr_name: {}", p.1);
                break;
            }
        }
        println!(" ==> rsp_ptr: {:?}", tsk.rsp_ptr);
        println!(" ==> rsp0: {:?}", tsk.rsp0);
        println!(" ==> break: {:?}", VirtAddr::new(tsk.program_break));
        println!(" ==== Regs ====");
        println!("   rbx = {:#x?}", tsk.state.rbx);
        println!("   rbp = {:#x?}", tsk.state.rbp);
        println!("   r12 = {:#x?}", tsk.state.r12);
        println!("   r13 = {:#x?}", tsk.state.r13);
        println!("   r14 = {:#x?}", tsk.state.r14);
        println!("   r15 = {:#x?}", tsk.state.r15);
        println!("   rsp = {:#x?}", tsk.state.rsp);
        println!("   rip = {:#x?}", tsk.state.rip);
		println!("   rsi = {:#x?}", tsk.state.rsi);
		println!(" == endps ==");
    }
    println!("~~ End CSI Report ~~");
}
