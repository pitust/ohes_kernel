#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![feature(exclusive_range_pattern)]
#![feature(lang_items)]
#![feature(unboxed_closures)]
#![test_runner(crate::main::test_runner)]
#![feature(box_syntax)]
#![reexport_test_harness_main = "test_main"]
#![allow(unused_imports)]
#![feature(naked_functions)]
#![feature(const_ptr_offset)]
#![feature(iter_advance_by)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(link_llvm_intrinsics)]

extern crate alloc;
extern crate faster_rlibc;
extern crate kmacros;
extern crate safety_here;

pub mod constants;
pub mod devices;
pub mod drive;
pub mod events;
pub mod exiting;
pub mod init;
pub mod interrupts;
pub mod io;
pub mod stack_canaries;
pub mod ksvc;
pub mod ksymmap;
pub mod main;
pub mod memory;
pub mod pci;
pub mod preempt;
pub mod prelude;
pub mod proc;
pub mod queue;
pub mod shell;
pub mod shittymutex;
pub mod task;
pub mod testing;
pub mod unwind;
pub mod userland;
