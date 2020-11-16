use crate::prelude::*;

pub mod debugcon;
pub mod kbdint_input;
pub mod multiboot_text;
pub mod multiboot_vga;
pub mod serial;

pub trait IODevice: Send + Sync {
    fn write_str(&mut self, _s: &str) {}
    fn write_bytes(&mut self, _s: &[u8]) {}
    fn read_chr(&mut self) -> Option<char> {
        None
    }
    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}

pub struct Mutlidev {
    dev: Vec<Box<dyn IODevice>>,
}
impl Mutlidev {
    pub fn new(d: Vec<Box<dyn IODevice>>) -> Mutlidev {
        Mutlidev { dev: d }
    }
    pub fn empty() -> Mutlidev {
        Mutlidev::new(vec![])
    }
}
impl IODevice for Mutlidev {
    fn write_str(&mut self, s: &str) {
        for i in self.dev.as_mut_slice() {
            i.write_str(s);
        }
    }

    fn read_chr(&mut self) -> Option<char> {
        for i in self.dev.as_mut_slice() {
            match i.read_chr() {
                Some(c) => return Some(c),
                None => {}
            }
        }
        None
    }

    // you can't seek a multidev
    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
