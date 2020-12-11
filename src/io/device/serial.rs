use crate::prelude::*;
use io::device::IODevice;

pub struct Serial {
    se: u16,
}
impl Serial {
    pub fn new(port: u16) -> Serial {
        unsafe {
            outb(port + 1, 0x00); // Disable all interrupts
            outb(port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(port + 1, 0x00); //                  (hi byte)
            outb(port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }

        Serial { se: port }
    }
}
impl IODevice for Serial {
    fn write_str(&mut self, s: &str) {
        for c in s.as_bytes() {
            while unsafe { inb(self.se + 5) & 0x20 } == 0 {}
            unsafe {
                outb(self.se, *c);
            }
        }
    }

    fn read_chr(&mut self) -> Option<char> {
        if unsafe { inb(self.se + 5) & 1 } != 0 {
            Some(unsafe { inb(self.se) } as char)
        } else {
            None
        }
    }

    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
