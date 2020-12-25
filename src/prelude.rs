pub use crate::shittymutex::Mutex;
pub use crate::*;
pub use crate::{
    _ezy_static, counter, dbg, dprint, dprintln, ezy_static, input, io::Printer, print, println,
    testing,
};
pub use alloc::format;
pub use alloc::{boxed::Box, collections::*, collections::*, string::*, vec, vec::Vec};
pub use core::sync::atomic::*;
pub use lazy_static::lazy_static;
pub use x86::io::*;
pub use x86_64::VirtAddr;
#[macro_export]
#[allow(non_upper_case_globals)]
macro_rules! ezy_static {
    { $name:ident, $type:ty, $init:expr } => {
        #[allow(non_upper_case_globals)]
        lazy_static! {
            pub static ref $name: Mutex<$type> = {
                Mutex::new($init)
            };
        }
    }
}

#[macro_export]
macro_rules! _ezy_static {
    { $name:ident, $type:ty, $init:expr } => {
        lazy_static! {
            static ref $name: Mutex<$type> = {
                Mutex::new($init)
            };
        }
    }
}

#[macro_export]
macro_rules! counter {
    ( $NAME: ident ) => {
        pub mod $NAME {
            use $crate::prelude::*;
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            pub fn get() -> usize {
                return COUNTER.load(Ordering::Relaxed);
            }
            pub fn inc() -> usize {
                return COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
            }
            pub fn dec() -> usize {
                return COUNTER.fetch_sub(1, Ordering::Relaxed) - 1;
            }
            pub fn addn(n: usize) -> usize {
                return COUNTER.fetch_add(n, Ordering::Relaxed) + n;
            }
            pub fn subn(n: usize) -> usize {
                return COUNTER.fetch_sub(n, Ordering::Relaxed) - n;
            }
            pub fn reset() -> usize {
                let old = get();
                COUNTER.store(0, Ordering::Relaxed);
                return old;
            }
        }
    }
}
