use crate::queue::ArrayQueue;
use crate::shittymutex::Mutex;
use crate::{dbg, print, println};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use lazy_static::lazy_static;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
pub mod cpio;
pub mod gpt;
pub mod ext2;
pub mod fat;
#[derive(Debug)]
pub struct Offreader {
    drive: Drive,
    offlba: u32,
    queue: ArrayQueue<u8>,
}

pub trait RODev {
    fn read_from(&mut self, lba: u32) -> Result<Vec<u8>, String>;
}

#[derive(Debug, Clone)]
pub struct Drive {
    data: Port<u16>,
    error: PortReadOnly<u8>,
    features: PortWriteOnly<u8>,
    sec_count: Port<u8>,
    lba_low: Port<u8>,
    lba_mid: Port<u8>,
    lba_high: Port<u8>,
    dhr: Port<u8>,
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,
    is_slave: bool,
}
impl Drive {
    pub unsafe fn new(is_slave: bool, base: u16, _base2: u16) -> Drive {
        Drive {
            data: Port::new(base),
            error: PortReadOnly::new(base + 1),
            features: PortWriteOnly::new(base + 1),
            sec_count: Port::new(base + 2),
            lba_low: Port::new(base + 3),
            lba_mid: Port::new(base + 4),
            lba_high: Port::new(base + 5),
            dhr: Port::new(base + 6),
            status: PortReadOnly::new(base + 7),
            command: PortWriteOnly::new(base + 7),
            // alt_status: PortReadOnly::new(base2),
            is_slave,
        }
    }
    pub fn get_offreader(&self) -> Offreader {
        Offreader {
            drive: self.clone(),
            offlba: 0,
            queue: ArrayQueue::new(1024),
        }
    }
}
impl RODev for Drive {
    fn read_from(&mut self, lba: u32) -> Result<Vec<u8>, String> {
        let mut vec = Vec::new();
        vec.reserve_exact(512);
        unsafe { vec.set_len(512) };
        unsafe {
            self.dhr.write(
                (0xE0 | ((lba >> 24) & 0x0F) | {
                    if self.is_slave {
                        0x10
                    } else {
                        0x00
                    }
                }) as u8,
            );
            self.features.write(0x00);
            self.sec_count.write(1);
            self.lba_low.write(lba as u8);
            self.lba_mid.write((lba >> 8) as u8);
            self.lba_high.write((lba >> 16) as u8);
            self.command.write(0x20);
        }
        while unsafe {
            let x: u8 = self.status.read();
            x & 0x8 != 0x8
        } {
            if unsafe { self.status.read() } & 1 == 1 {
                return Err(
                    "Drive read error: ".to_string() + &unsafe { self.error.read() }.to_string()
                );
            }
        }
        for i in 0..256 {
            unsafe {
                let val = self.data.read();
                let hi = (val >> 8) as u8;
                let lo = (val & 0xff) as u8;
                vec[i * 2 + 0] = lo;
                vec[i * 2 + 1] = hi;
            }
        }
        Ok(vec)
    }
}
impl Offreader {
    pub fn offset(&self, off: u32) -> Result<Offreader, String> {
        let mut nor = Offreader {
            drive: self.drive.clone(),
            offlba: self.offlba,
            queue: self.queue.clone(),
        };
        for _i in 0..off {
            if nor.queue.count == 0 {
                nor.read_sector()?
            }
            nor.queue.pop();
        }
        Ok(nor)
    }
    fn read_sector(&mut self) -> Result<(), String> {
        let data = self.drive.read_from(self.offlba)?;
        self.offlba += 1;
        for i in 0..512 {
            self.queue.push(data[i]);
        }
        Ok(())
    }
    pub fn read_consume(&mut self, n: u64) -> Result<Vec<u8>, String> {
        let mut o = vec![];
        for _i in 0..n {
            if self.queue.count == 0 {
                self.read_sector()?;
            }
            let e = self.queue.pop();
            o.push(*e);
        }
        Ok(o)
    }
}
