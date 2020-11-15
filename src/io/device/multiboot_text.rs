use crate::prelude::*;
use font8x8::UnicodeFonts;
use io::device::IODevice;
use multiboot2::FramebufferField;

pub struct MultibootText {
    addr: u64,
    w: u32,
    h: u32,
    x_pos: u32,
}
#[repr(C)]
pub struct VGATextCell {
    chr: u8,
    cc: u8,
}
impl MultibootText {
    pub fn new(tag: multiboot2::FramebufferTag<'static>) -> MultibootText {
        match tag.buffer_type {
            multiboot2::FramebufferType::Indexed { palette: _ } => {
                panic!("No support for indexed FB")
            }
            multiboot2::FramebufferType::RGB {
                red: _,
                green: _,
                blue: _,
            } => panic!("Create a MultibootVGA please"),
            multiboot2::FramebufferType::Text => {
                // Text FB
            }
        };
        MultibootText {
            addr: tag.address,
            w: tag.width,
            h: tag.height,
            x_pos: 0,
        }
    }
    pub fn put_char_at(&mut self, x_pos: u32, chr: char) -> usize {
        unsafe {
            (*(self.addr as *mut VGATextCell).offset((x_pos + ((self.h - 1) * self.w)) as isize))
                .chr = chr as u8;
            (*(self.addr as *mut VGATextCell).offset((x_pos + ((self.h - 1) * self.w)) as isize))
                .cc = 0xf;
        }
        1
    }
    pub fn set_cursor_pos(&self, x: u16, y: u16) {
        let pos = y * (self.w as u16) + x;

        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }
    pub fn putc(&mut self, chr: char) {
        let mut off = self.x_pos;
        off = match chr {
            '\n' => {
                self.new_line();
                0
            }
            '\x20'..'\x7f' => off + self.put_char_at(off, chr) as u32,
            '\x08' => {
                self.put_char_at(off - 1, ' ');
                off - 1
            }
            _ => off,
        };
        self.x_pos = off;
        self.update_loc();
    }
    pub fn new_line(&mut self) {
        self.scroll_up()
    }
    pub fn scroll_up(&mut self) {
        let o = self.addr as *mut u8;

        self.x_pos = 0;
        unsafe {
            faster_rlibc::fastermemcpy(
                o,
                o.offset((self.w * 2) as isize),
                (self.w * 2 * (self.h - 1)) as usize,
            );
            faster_rlibc::fastermemset(
                o.offset((self.w * 2 * (self.h - 1)) as isize),
                0,
                (self.w * 2) as usize,
            );
        }
        self.update_loc();
    }
    fn update_loc(&self) {
        self.set_cursor_pos(self.x_pos as u16, (self.h - 1) as u16);
    }
}
impl IODevice for MultibootText {
    fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
    }

    fn read_chr(&mut self) -> Option<char> {
        None
    }

    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
