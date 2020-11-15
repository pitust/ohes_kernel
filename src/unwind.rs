use crate::prelude::*;
use core::ffi::c_void;
use gimli::{DebugLineOffset, LittleEndian};
use x86_64::VirtAddr;
use xmas_elf::sections::SectionData;
use xmas_elf::{self, symbol_table::Entry};
extern "C" {
    pub fn __register_frame(__frame: *mut c_void, __size: usize);
}
#[no_mangle]
unsafe extern "C" fn strlen(mut s: *mut u8) -> usize {
    let mut i = 0;
    while *s != 0 {
        i += 1;
        s = s.offset(1);
    }
    i
}
#[no_mangle]
unsafe extern "C" fn abort() -> ! {
    panic!("C abort");
}
pub fn basic_unmangle(s: alloc::string::String) -> alloc::string::String {
    if !s.starts_with("_Z") {
        return s;
    }
    let old = cpp_demangle::Symbol::new(s)
        .unwrap()
        .to_string()
        .replace("..", "::")
        .replace("$LT$", "<")
        .replace("$GT$", ">")
        .replace("$u20$", " ")
        .replace("$u7b$", "{")
        .replace("$u7d$", "}");
    let mut vec: Vec<&str> = old.split("::").collect();
    vec.pop();
    return vec.join("::");
}
ezy_static! { SYMBOL_TABLE, BTreeMap<u64, String>, BTreeMap::<u64, String>::new() }
pub fn line_tweaks(debug_line: &[u8]) {
    let d = gimli::DebugLine::new(debug_line, LittleEndian);
    let data = d.program(DebugLineOffset(0), 8, None, None).unwrap();
    println!("{:?}", data.header());
}
pub fn register_module(kernel: &[u8]) {
    let k = xmas_elf::ElfFile::new(kernel).unwrap();
    let symbols = SYMBOL_TABLE.get();
    for s in k.section_iter() {
        match s.get_name(&k) {
            Ok(".eh_frame") => {
                // yay!
                // addr is s.address()
                // len is s.size()
                unsafe {
                    __register_frame(s.address() as *mut c_void, s.size() as usize);
                }
            }
            Ok(".symtab") => match s.get_data(&k).unwrap() {
                SectionData::SymbolTable64(st) => {
                    for e in st {
                        let name = e.get_name(&k).unwrap();
                        let addr = e.value();
                        symbols.insert(addr, name.to_string());
                    }
                }
                _ => {}
            },
            Ok(".debug_line") => match s.get_data(&k).unwrap() {
                SectionData::Undefined(raw_data) => {
                    line_tweaks(raw_data);
                }
                _ => {}
            },
            Ok(s) => {
                println!("S: {}", s);
            }
            _ => {}
        }
    }
}
pub fn backtrace() {
    let symbols = SYMBOL_TABLE.get();
    unsafe {
        trace(&mut |f| {
            let mut addr = ((f.ip() as u64) >> 4) << 4;
            if addr > 0xf00000000 {
                loop {
                    if addr == 0 {
                        break;
                    }
                    match symbols.get(&addr) {
                        Some(_) => {
                            break;
                        }
                        None => {
                            addr -= 1;
                        }
                    }
                }
            }
            ksymmap::addr_fmt(
                f.ip() as u64,
                match symbols.get(&addr) {
                    Some(sym) => Some(basic_unmangle(sym.clone())),
                    None => None,
                },
            );

            true
        });
    }
}

pub enum Frame {
    Raw(*mut uw::_Unwind_Context),
    Cloned {
        ip: *mut c_void,
        sp: *mut c_void,
        symbol_address: *mut c_void,
    },
}

// With a raw libunwind pointer it should only ever be access in a readonly
// threadsafe fashion, so it's `Sync`. When sending to other threads via `Clone`
// we always switch to a version which doesn't retain interior pointers, so we
// should be `Send` as well.
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    pub fn ip(&self) -> *mut c_void {
        let ctx = match *self {
            Frame::Raw(ctx) => ctx,
            Frame::Cloned { ip, .. } => return ip,
        };
        unsafe { uw::_Unwind_GetIP(ctx) as *mut c_void }
    }

    pub fn sp(&self) -> *mut c_void {
        match *self {
            Frame::Raw(ctx) => unsafe { uw::get_sp(ctx) as *mut c_void },
            Frame::Cloned { sp, .. } => sp,
        }
    }

    pub fn symbol_address(&self) -> *mut c_void {
        if let Frame::Cloned { symbol_address, .. } = *self {
            return symbol_address;
        }
        unsafe { uw::_Unwind_FindEnclosingFunction(self.ip()) }
    }

    pub fn module_base_address(&self) -> Option<*mut c_void> {
        None
    }
}

impl Clone for Frame {
    fn clone(&self) -> Frame {
        Frame::Cloned {
            ip: self.ip(),
            sp: self.sp(),
            symbol_address: self.symbol_address(),
        }
    }
}

#[inline(always)]
pub unsafe fn trace(mut cb: &mut dyn FnMut(&Frame) -> bool) {
    uw::_Unwind_Backtrace(trace_fn, &mut cb as *mut _ as *mut _);

    extern "C" fn trace_fn(
        ctx: *mut uw::_Unwind_Context,
        arg: *mut c_void,
    ) -> uw::_Unwind_Reason_Code {
        let cb = unsafe { &mut *(arg as *mut &mut dyn FnMut(&Frame) -> bool) };
        let cx = Frame::Raw(ctx);

        let keep_going = cb(&cx);

        if keep_going {
            uw::_URC_NO_REASON
        } else {
            uw::_URC_FAILURE
        }
    }
}

/// Unwind library interface used for backtraces
///
/// Note that dead code is allowed as here are just bindings
/// iOS doesn't use all of them it but adding more
/// platform-specific configs pollutes the code too much
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod uw {
    pub use self::_Unwind_Reason_Code::*;

    use core::ffi::c_void;

    #[repr(C)]
    pub enum _Unwind_Reason_Code {
        _URC_NO_REASON = 0,
        _URC_FOREIGN_EXCEPTION_CAUGHT = 1,
        _URC_FATAL_PHASE2_ERROR = 2,
        _URC_FATAL_PHASE1_ERROR = 3,
        _URC_NORMAL_STOP = 4,
        _URC_END_OF_STACK = 5,
        _URC_HANDLER_FOUND = 6,
        _URC_INSTALL_CONTEXT = 7,
        _URC_CONTINUE_UNWIND = 8,
        _URC_FAILURE = 9, // used only by ARM EABI
    }

    pub enum _Unwind_Context {}

    pub type _Unwind_Trace_Fn =
        extern "C" fn(ctx: *mut _Unwind_Context, arg: *mut c_void) -> _Unwind_Reason_Code;

    extern "C" {
        pub fn _Unwind_Backtrace(
            trace: _Unwind_Trace_Fn,
            trace_argument: *mut c_void,
        ) -> _Unwind_Reason_Code;

        // available since GCC 4.2.0, should be fine for our purpose
        #[cfg(all(
            not(all(target_os = "android", target_arch = "arm")),
            not(all(target_os = "freebsd", target_arch = "arm")),
            not(all(target_os = "linux", target_arch = "arm"))
        ))]
        pub fn _Unwind_GetIP(ctx: *mut _Unwind_Context) -> usize;

        #[cfg(all(
            not(all(target_os = "android", target_arch = "arm")),
            not(all(target_os = "freebsd", target_arch = "arm")),
            not(all(target_os = "linux", target_arch = "arm"))
        ))]
        pub fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void;

        #[cfg(all(
            not(all(target_os = "android", target_arch = "arm")),
            not(all(target_os = "freebsd", target_arch = "arm")),
            not(all(target_os = "linux", target_arch = "arm"))
        ))]
        // This function is a misnomer: rather than getting this frame's
        // Canonical Frame Address (aka the caller frame's SP) it
        // returns this frame's SP.
        //
        // https://github.com/libunwind/libunwind/blob/d32956507cf29d9b1a98a8bce53c78623908f4fe/src/unwind/GetCFA.c#L28-L35
        #[link_name = "_Unwind_GetCFA"]
        pub fn get_sp(ctx: *mut _Unwind_Context) -> usize;
    }
}
