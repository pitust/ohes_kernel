#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        x86::io::outl(0xf4, exit_code as u32);
    }
    halt();
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn exit() -> ! {
    if super::constants::should_fini_exit() {
        exit_qemu(QemuExitCode::Success);
    }
    if super::constants::should_fini_wait() {
        halt();
    }
    panic!("The constants are invalid")
}
pub fn exit_fail() -> ! {
    if super::constants::should_fini_exit() {
        exit_qemu(QemuExitCode::Failed);
    }
    if super::constants::should_fini_wait() {
        halt();
    }
    panic!("The constants are invalid")
}
