use crate::prelude::*;
use io::device::IODevice;

pub struct DebugCon {
    pub port: u16,
}
impl IODevice for DebugCon {
    fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            unsafe {
                outb(self.port, c as u8);
            }
        }
    }

    fn read_chr(&mut self) -> Option<char> {
        None
    }

    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
