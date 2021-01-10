use crate::prelude::*;
pub struct Repe {
    pub s: String,
}
impl io::device::IODevice for Repe {
    fn write_str(&mut self, _s: &str) {}

    fn read_chr(&mut self) -> Option<char> {
        if self.s.len() != 0 {
            let c = self.s.chars().next().unwrap();
            self.s = self.s.split_off(1);
            Some(c)
        } else {
            None
        }
    }

    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
