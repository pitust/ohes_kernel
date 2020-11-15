use crate::prelude::*;
use font8x8::UnicodeFonts;
use io::device::IODevice;
use multiboot2::FramebufferField;

pub struct MultibootVGA {
    addr: u64,
    w: u32,
    h: u32,
    x_pos: u32,
    y_pos: u32,
    font: fontdue::Font,
}
impl MultibootVGA {
    pub fn new(tag: multiboot2::FramebufferTag<'static>) -> MultibootVGA {
        match tag.buffer_type {
            multiboot2::FramebufferType::Indexed { palette: _ } => {
                panic!("No support for indexed FB")
            }
            multiboot2::FramebufferType::RGB {
                red: _,
                green: _,
                blue: _,
            } => {}
            multiboot2::FramebufferType::Text => {
                // Text FB
                panic!("Create a MultibootText instead");
            }
        };
        let font = include_bytes!("../../../fonts/roboto.ttf") as &[u8];
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        MultibootVGA {
            addr: tag.address,
            w: tag.width,
            h: tag.height,
            x_pos: 0,
            y_pos: 0,
            font,
        }
    }
    pub fn put_pxl_at(&mut self, x: u32, y: u32, is_on: bool) {
        let off = 4 * (self.w * y + x);

        unsafe {
            if is_on {
                *(self.addr as *mut u8).offset(off as isize).offset(2) =
                    io::CR.load(Ordering::Relaxed) as u8;
                *(self.addr as *mut u8).offset(off as isize).offset(1) =
                    io::CG.load(Ordering::Relaxed) as u8;
                *(self.addr as *mut u8).offset(off as isize).offset(0) =
                    io::CB.load(Ordering::Relaxed) as u8;
            } else {
                *(self.addr as *mut u32).offset(off as isize) = 0;
            }
        };
    }
    pub fn put_char_at(&mut self, x_pos: u32, chr: char) -> usize {
        let (m, r) = self.font.rasterize(chr, 8.0);
        for x in 0..m.width {
            for y in 0..m.height {
                let v = r[y * m.width + x];
                self.put_pxl_at(x as u32 + x_pos, y as u32 + self.h - 8, v < 128);
                if x as u32 + x_pos > self.w {
                    self.new_line()
                }
            }
        }
        m.width
    }
    pub fn putc(&mut self, chr: char) {
        let mut off = self.x_pos;
        off = match chr {
            '\n' => {
                self.new_line();
                0
            }
            '\x20'..'\x7f' => off + self.put_char_at(off, chr) as u32,
            _ => off,
        };
        self.x_pos = off
    }
    pub fn new_line(&mut self) {
        self.scroll_up()
    }
    pub fn scroll_up(&mut self) {
        let o = self.addr as *mut u8;

        self.y_pos -= 16;
        self.x_pos = 0;
        unsafe {
            faster_rlibc::fastermemcpy(
                o,
                o.offset((32 * self.w) as isize),
                (self.w * 4 * (self.h - 16)) as usize,
            );
        }
    }
}
impl IODevice for MultibootVGA {
    fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            // TODO: multiboot VGA
            self.put_char_at(0, c);
        }
    }

    fn read_chr(&mut self) -> Option<char> {
        None
    }

    fn seek(&mut self, _to: usize) -> Option<()> {
        None
    }
}
