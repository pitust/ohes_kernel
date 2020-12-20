use crate::prelude::*;

ezy_static! { CANARIES, Vec<(VirtAddr, String, u64)>, vec![] }


pub fn add_canary(p: VirtAddr, s: String, size: u64) {
    let q = unsafe { p.as_ptr::<u8>() };
    unsafe {
        *(q as *mut u64) = 0xdeadbeef;
    }
    CANARIES.get().push((p, s.clone(), size));
}

extern {
    #[link_name = "llvm.returnaddress"]
    fn return_address(level: i32) -> *const u8;
}


pub fn stk_chk() {
    for c in CANARIES.get() {
        let q = unsafe { c.0.as_ptr::<u8>() };
        let stka = unsafe {
            *(q as *mut u64)
        };
        if stka != 0xdeadbeef {
            println!("=={}== ERROR: AddressSanitizer stack-overflow on address {:p} at pc {:p} stack {}", preempt::CURRENT_TASK.pid, q, unsafe { return_address(0) }, c.1);
            println!("SMASHED of size 8 at {:p} (now is {:#x?})", q, stka);
            println!("=={}== ABORTING", preempt::CURRENT_TASK.pid);
        }
    }
}