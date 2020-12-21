use crate::constants;
use crate::shittymutex::Mutex;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::{
    cell::Cell,
    fmt::{Arguments, Result, Write},
};
use core::{file, line, stringify};
use core::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
    task::Poll,
};
use font8x8::UnicodeFonts;
use lazy_static::lazy_static;

use self::device::IODevice;

pub static X_POS: AtomicUsize = AtomicUsize::new(0);
pub static Y_POS: AtomicUsize = AtomicUsize::new(0);
pub static CR: AtomicUsize = AtomicUsize::new(255);
pub static CG: AtomicUsize = AtomicUsize::new(255);
pub static CB: AtomicUsize = AtomicUsize::new(255);

pub mod device;

pub enum MaybeInitDevice {
    GotMman(
        alloc::vec::Vec<Box<dyn device::IODevice>>,
        alloc::vec::Vec<Box<dyn device::IODevice>>,
    ),
    NoMman,
}

impl MaybeInitDevice {
    pub fn force_maybeinitdev(&mut self) -> &mut Self {
        self
    }
}

pub struct Printer;
pub struct DbgPrinter;
lazy_static! {
    pub static ref IO_DEVS: Mutex<MaybeInitDevice> = Mutex::new(MaybeInitDevice::NoMman);
}

pub fn proper_init_for_iodevs(mbstruct: &'static multiboot2::BootInformation) {
    // we got mman now! let's get i/o subsystem fixed
    // first, parse out kcmdline
    let kcmdline = mbstruct.command_line_tag().unwrap().command_line();
    let mut devs: alloc::vec::Vec<Box<dyn device::IODevice>> = alloc::vec![];
    let mut ddevs: alloc::vec::Vec<Box<dyn device::IODevice>> = alloc::vec![];
    let mut nid = false;
    for op in kcmdline.split(" ") {
        if op.starts_with("debugcon=") {
            let debugcon_addr = op.split_at(8).1.parse().unwrap();
            devs.push(box device::debugcon::DebugCon {
                port: debugcon_addr,
            });
            if nid {
                ddevs.push(box device::debugcon::DebugCon {
                    port: debugcon_addr,
                });
            }
        }
        if op.starts_with("serial=") {
            let debugcon_addr = op.split_at(7).1.parse().unwrap();
            devs.push(box device::serial::Serial::new(debugcon_addr));
            if nid {
                ddevs.push(box device::serial::Serial::new(debugcon_addr));
            }
        }
        if op.starts_with("input-txt=") {
            let dat = op.split_at(10).1;
            devs.push(box device::virt::Repe { s: dat.to_string() + "\n" });
        }
        if op == "default_serial" {
            devs.push(box device::serial::Serial::new(0x3F8));
            if nid {
                ddevs.push(box device::debugcon::DebugCon { port: 0x402 });
            }
        }
        if op == "default_debugcon" {
            devs.push(box device::debugcon::DebugCon { port: 0x402 });
            if nid {
                ddevs.push(box device::debugcon::DebugCon { port: 0x402 });
            }
        }
        if op == "textvga" {
            devs.push(box device::multiboot_text::MultibootText::new(
                mbstruct.framebuffer_tag().unwrap(),
            ));
            if nid {
                ddevs.push(box device::multiboot_text::MultibootText::new(
                    mbstruct.framebuffer_tag().unwrap(),
                ));
            }
        }
        if op == "graphicz" {
            devs.push(box device::multiboot_vga::MultibootVGA::new(
                mbstruct.framebuffer_tag().unwrap(),
            ));
            if nid {
                ddevs.push(box device::multiboot_vga::MultibootVGA::new(
                    mbstruct.framebuffer_tag().unwrap(),
                ));
            }
        }
        if op == "kbdint" {
            devs.push(box device::kbdint_input::KbdInt {});
            if nid {
                ddevs.push(box device::kbdint_input::KbdInt {});
            }
        }
        if op == "debug:" {
            nid = true;
        } else {
            nid = false;
        }
    }
    *IO_DEVS.get() = MaybeInitDevice::GotMman(devs, ddevs);
    Printer
        .write_fmt(format_args!("Done kernel commandline: {}\n", kcmdline))
        .unwrap();
    unsafe {
        log::set_logger_racy(&KLogImpl).unwrap();
    }
    crate::println!("{}", log::max_level());
    log::set_max_level(LevelFilter::Trace);
    log::info!("Test");
}

impl Printer {
    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        CR.store(r as usize, Ordering::SeqCst);
        CG.store(g as usize, Ordering::SeqCst);
        CB.store(b as usize, Ordering::SeqCst);
        // TODO: move this to io::device::serial

        // SERIAL
        //     .get()
        //     .write_fmt(format_args!("\x1b[38;2;{};{};{}m", r, g, b))
        //     .expect("Printing to serial failed");
    }
    pub fn clear_screen(&mut self) {
        panic!("Clear Screen in legacy io printer");
    }
    pub fn scroll_up(&mut self) {
        // TODO: remove cuz autoscroll
        panic!("Scroll Up in legacy io printer");
    }

    pub fn newline(&mut self) {
        panic!("newline in legacy io printer");
    }
}

impl Write for Printer {
    fn write_str(&mut self, s: &str) -> Result {
        match IO_DEVS.get().force_maybeinitdev() {
            MaybeInitDevice::GotMman(mmaned, _dbgdevs) => {
                for mm in mmaned {
                    mm.write_str(s);
                }
            }
            MaybeInitDevice::NoMman => {
                if crate::constants::should_debug_log() {
                    for c in s.chars() {
                        unsafe {
                            x86::io::outb(0x402, c as u8);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
impl Write for DbgPrinter {
    fn write_str(&mut self, s: &str) -> Result {
        match IO_DEVS.get().force_maybeinitdev() {
            MaybeInitDevice::GotMman(_mmaned, dbgdevs) => {
                for mm in dbgdevs {
                    mm.write_str(s);
                }
            }
            MaybeInitDevice::NoMman => {
                if crate::constants::should_debug_log() {
                    for c in s.chars() {
                        unsafe {
                            x86::io::outb(0x402, c as u8);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::print_out(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! dprint {
    ($($arg:tt)*) => ($crate::io::dprint_out(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dprintln {
    () => ($crate::dprint!("\n"));
    ($($arg:tt)*) => ($crate::dprint!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { $crate::dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
enum EHS {
    None,
    Bracket,
    Value,
}
pub struct ReadLineFuture<
    Ta: FnMut<(), Output = Option<String>> + Sized,
    Tb: FnMut<(), Output = Option<String>> + Sized,
    Tc: FnMut<(String,), Output = Option<String>> + Sized,
> {
    text: alloc::boxed::Box<String>,
    finito: bool,
    arrow_up: Ta,
    arrow_down: Tb,
    complete: Tc,
    escape_handling_step: EHS,
}
impl<
        Ta: FnMut<(), Output = Option<String>> + Sized + core::marker::Unpin,
        Tb: FnMut<(), Output = Option<String>> + Sized + core::marker::Unpin,
        Tc: FnMut<(String,), Output = Option<String>> + Sized + core::marker::Unpin,
    > ReadLineFuture<Ta, Tb, Tc>
{
    pub fn handle_c(&mut self, k: char) -> Poll<String> {
        if self.escape_handling_step == EHS::Bracket {
            // assert_eq!(k, '[', "Wrong escape sent over serial");
            self.escape_handling_step = EHS::Value;
            return Poll::Pending;
        }
        if self.escape_handling_step == EHS::Value {
            let result = match k {
                'A' => {
                    // arrow up
                    (self.arrow_up)()
                }
                'B' => {
                    // arrow down
                    (self.arrow_down)()
                }
                _ => None,
            };

            match result {
                Some(str) => {
                    let mut el = self.text.as_ref().to_owned();
                    while el.len() != 0 {
                        el.remove(el.len() - 1);
                        print!("\x08 \x08");
                    }
                    *self.text = str.clone();
                    print!("{}", self.text);
                }
                None => {}
            }
            self.escape_handling_step = EHS::None;
            return Poll::Pending;
        }
        if k == '\r' || k == '\n' {
            println!();
            self.finito = true;
            return Poll::Ready(String::from(self.text.as_str()));
        } else if k == '\x7f' || k == '\x08' {
            let mut el = self.text.as_ref().to_owned();
            if el.len() == 0 {
                return Poll::Pending;
            }
            el.remove(el.len() - 1);
            print!("\x08 \x08");
            *self.text = el;
            return Poll::Pending;
        } else if k == '\x1b' {
            // TODO: fix this
            self.escape_handling_step = EHS::Bracket;
            // let c = SERIAL.get().receive() as char;
            // let result = match c {
            //     'A' => {
            //         // arrow up
            //         (self_ref.arrow_up)()
            //     }
            //     'B' => {
            //         // arrow down
            //         (self_ref.arrow_down)()
            //     }
            //     _ => None,
            // };

            // match result {
            //     Some(str) => {
            //         let mut el = self_ref.text.as_ref().to_owned();
            //         while el.len() != 0 {
            //             el.remove(el.len() - 1);
            //             print!("\x08");
            //         }
            //         *self_ref.text = str.clone();
            //         print!("{}", self_ref.text);
            //     }
            //     None => {}
            // }
            return Poll::Pending;
        } else if k == '\t' {
            let result = (self.complete)(self.text.to_string());
            match result {
                Some(str) => {
                    let mut el = self.text.as_ref().to_owned();
                    while el.len() != 0 {
                        el.remove(el.len() - 1);
                        print!("\x08");
                    }
                    *self.text = str.clone();
                    print!("{}", self.text);
                }
                None => {}
            }
        } else {
            print!("{}", k);
            *self.text = self.text.as_ref().to_owned() + &String::from(k);
        }
        return Poll::Pending;
    }
}
impl<
        Ta: FnMut<(), Output = Option<String>> + Sized + core::marker::Unpin,
        Tb: FnMut<(), Output = Option<String>> + Sized + core::marker::Unpin,
        Tc: FnMut<(String,), Output = Option<String>> + Sized + core::marker::Unpin,
    > Future for ReadLineFuture<Ta, Tb, Tc>
{
    type Output = String;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _: &mut core::task::Context<'_>,
    ) -> core::task::Poll<<Self as core::future::Future>::Output> {
        let self_ref = self.get_mut();
        if self_ref.finito {
            return Poll::Ready(String::from(self_ref.text.as_str()));
        }
        let k = (|| match IO_DEVS.get().force_maybeinitdev() {
            MaybeInitDevice::GotMman(d, _d2) => {
                for k in d {
                    match k.read_chr() {
                        Some(k) => {
                            return Some(k);
                        }
                        None => {}
                    }
                }
                None
            }
            MaybeInitDevice::NoMman => None,
        })();
        // println!("{:?}", k);
        match k {
            Some(k) => match self_ref.handle_c(k) {
                Poll::Ready(r) => {
                    return Poll::Ready(r);
                }
                Poll::Pending => {}
            },
            None => {}
        }

        return Poll::Pending;
    }
}
pub fn read_line<
    Ta: FnMut<(), Output = Option<String>> + Sized,
    Tb: FnMut<(), Output = Option<String>> + Sized,
    Tc: FnMut<(String,), Output = Option<String>> + Sized,
>(
    arrow_up: Ta,
    arrow_down: Tb,
    complete: Tc,
) -> ReadLineFuture<Ta, Tb, Tc> {
    ReadLineFuture {
        text: alloc::boxed::Box::from(String::from("")),
        finito: false,
        arrow_up,
        arrow_down,
        complete,
        escape_handling_step: EHS::None,
    }
}

#[macro_export]
macro_rules! input {
    () => {
        $crate::io::read_line(
            || {
                return None;
            },
            || {
                return None;
            },
            |s: String| {
                return None;
            },
        );
    };
    ($arrowup: expr, $arrowdown: expr, $complete: expr) => {
        $crate::io::read_line($arrowup, $arrowdown, $complete);
    };
}

#[doc(hidden)]
pub fn print_out(args: Arguments) {
    Printer.write_fmt(args).expect("Write failed");
}

#[doc(hidden)]
pub fn dprint_out(args: Arguments) {
    DbgPrinter.write_fmt(args).expect("Write failed");
}

use log::{Level, LevelFilter, Metadata, Record};

struct KLogImpl;

impl log::Log for KLogImpl {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let o = match record.level() {
            Level::Error => "\x1b[32mERROR",
            Level::Warn => "\x1b[33mWARN",
            Level::Info => "\x1b[38mINFO",
            Level::Debug => "\x1b[30mDEBUG",
            Level::Trace => "\x1b[30;2mTRACE",
        };
        println!("{} {}\x1b[0m", o, record.args());
    }

    fn flush(&self) {}
}

#[no_mangle]
pub fn out(s: &str) {
    print!("{}", s);
}
