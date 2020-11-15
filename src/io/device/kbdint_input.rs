use crate::prelude::*;
use io::device::IODevice;

pub struct KbdInt {}
impl IODevice for KbdInt {
    fn write_str(&mut self, _s: &str) {}

    fn read_chr(&mut self) -> Option<char> {
        match task::keyboard::KEY_QUEUE.get().unwrap().pop() {
            Ok(k) => match k {
                pc_keyboard::DecodedKey::RawKey(_k) => None,
                pc_keyboard::DecodedKey::Unicode(c) => Some(c),
            },
            Err(_) => None,
        }
    }

    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
