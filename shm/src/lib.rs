#![no_std]

pub mod thread_destructor {
    pub fn register(t: *mut u8, dtor: unsafe extern "C" fn(*mut u8)) {}
}
pub mod config {
    pub fn default_oom_handler() {}
    pub const OS_MEMTRIM_LIMIT: usize = 0x1000000;
    pub const OS_MEMTRIM_WORTHY: usize = 0;
    pub const LOCAL_MEMTRIM_STOP: usize = 1024;
    pub const LOCAL_MEMTRIM_LIMIT: usize = 1024;
    pub const FRAGMENTATION_SCALE: usize = 10;

    pub fn extra_brk(size: usize) -> usize {
        // TODO: Tweak this.
        /// The BRK multiplier.
        ///
        /// Alignment
        const MULTIPLIER: usize = 1;

        (size + MULTIPLIER - 1) / MULTIPLIER * MULTIPLIER - size
    }
    /// Canonicalize a fresh allocation.
    ///
    /// The return value specifies how much _more_ space is requested to the fresh allocator.
    // TODO: Move to shim.
    #[inline]
    pub fn extra_fresh(size: usize) -> usize {
        /// The multiplier.
        ///
        /// The factor determining the linear dependence between the minimum segment, and the acquired
        /// segment.
        const MULTIPLIER: usize = 2;
        /// The minimum extra size to be BRK'd.
        const MIN_EXTRA: usize = 64;
        /// The maximal amount of _extra_ bytes.
        const MAX_EXTRA: usize = 1024;

        core::cmp::max(MIN_EXTRA, core::cmp::min(MULTIPLIER * size, MAX_EXTRA))
    }
}
pub mod syscalls {

    extern "C" {
        pub fn brk(to: *const u8) -> *mut u8;
    }
}
