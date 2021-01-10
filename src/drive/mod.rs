use crate::prelude::*;
use core::convert::TryInto;
use queue::ArrayQueue;
use x86_64::{
    instructions::port::{Port, PortReadOnly, PortWriteOnly},
    VirtAddr,
};
pub mod cpio;
pub mod ext2;
pub mod fat;
pub mod gpt;
#[derive(Debug)]
pub struct Offreader {
    drive: Drive,
    offlba: u32,
    queue: ArrayQueue<u8>,
}

#[derive(Debug, Clone)]
pub struct ReadOp {
    pub data: Vec<u8>,
    pub from: u64,
}

pub trait RODev {
    fn read_from(&mut self, lba: u32) -> Result<Vec<u8>, String>;
    fn read_unaligned(&mut self, addr: u64, len: u64) -> Result<Vec<u8>, String> {
        let mut q = self.read_from((addr / 512).try_into().unwrap())?;
        for i in 0..((len + 511) / 512) {
            q.extend(self.read_from(((addr / 512) + 1 + i).try_into().unwrap())?);
        }

        let mut q = q.split_off((addr % 512).try_into().unwrap());
        q.truncate(len as usize);
        Ok(q)
    }
    fn vector_read_ranges(&mut self, ops: &mut [(u64, u64)]) -> Vec<u8> {
        let mut p = vec![];
        for op in ops {
            p.extend(self.read_unaligned(op.0, op.1).unwrap());
        }
        p
    }
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

pub struct SickCustomDev {}
#[repr(C)]
#[derive(Debug, Clone)]
struct EDRPType {
    addr: u64,
    len_or_count: u64,
    off: u64,
    isread: u64,
}

static mut EDRP: EDRPType = EDRPType {
    addr: 0,
    len_or_count: 0,
    off: 0,
    isread: 0,
};
impl SickCustomDev {
    fn edrp_do_read() {
        let val = unsafe { &EDRP as *const EDRPType as u64 };
        assert!(
            val < (1 << 32),
            "EDRP is above 4GiB (is the kernel too large?)"
        );
        let val = val as u32;
        // do the read!
        unsafe {
            x86::io::outl(0xff00, val);
        }
    }
    fn edrp_do_vectored() {
        let val = unsafe { &EDRP as *const EDRPType as u64 };
        assert!(
            val < (1 << 32),
            "EDRP is above 4GiB (is the kernel too large?)"
        );
        let val = val as u32;
        // do the read!
        unsafe {
            x86::io::outl(0xff04, val);
        }
    }
}
impl RODev for SickCustomDev {
    fn read_from(&mut self, lba: u32) -> Result<Vec<u8>, String> {
        let edv = (lba as u64) * 512;
        let re = unsafe { &mut EDRP };
        let p: Vec<u8> = vec![0; 512];
        re.addr = memory::convpc(p.as_ptr());
        re.isread = 1;
        re.len_or_count = 512;
        re.off = edv;
        SickCustomDev::edrp_do_read();
        Ok(p)
    }
    fn read_unaligned(&mut self, addr: u64, len: u64) -> Result<Vec<u8>, String> {
        let edv = addr;
        let re = unsafe { &mut EDRP };
        let p: Vec<u8> = [0u8].repeat(len as usize);
        re.addr = memory::convpc(p.as_ptr());
        re.isread = 1;
        re.len_or_count = len;
        re.off = edv;
        if addr == 0x80c7200 {
            loop {}
        }
        SickCustomDev::edrp_do_read();
        Ok(p)
    }
    fn vector_read_ranges(&mut self, ops: &mut [(u64, u64)]) -> Vec<u8> {
        let edrp = unsafe { &mut EDRP };
        let mem = [0u8].repeat(ops.iter().map(|a| a.1).sum::<u64>() as usize);
        let mut sztot = 0;
        let mut arrz: Vec<u64> = vec![];
        let mut kepe: Vec<Box<EDRPType>> = vec![];
        for op in ops {
            let addr = memory::convpc(unsafe { mem.as_ptr().offset(sztot) });
            let q = box EDRPType {
                addr,
                len_or_count: op.1,
                off: op.0,
                isread: 1,
            };
            arrz.push(memory::convpc(q.as_ref() as *const EDRPType));
            kepe.push(q);
            sztot += op.1 as isize;
        }
        //phptr<EDRP>[]
        edrp.addr = memory::convpc(arrz.as_ptr());
        edrp.len_or_count = arrz.len() as u64;
        SickCustomDev::edrp_do_vectored();
        drop(kepe);

        mem
    }
    // fn vector_read_ranges(&mut self, ops: &mut [(u64, u64)]) -> Vec<u8> {
    //     let mem = [0u8].repeat(ops.iter().map(|a| a.1).sum::<u64>() as usize);
    //     let mut q = vec![];
    //     for op in ops {
    //         q.push(box EDRPType {
    //             addr: memory::translate(VirtAddr::new(op.data.as_ptr() as u64))
    //                 .unwrap()
    //                 .as_u64(),
    //             len_or_count: op.data.len() as u64,
    //             off: op.from,
    //             isread: 1,
    //         })
    //     }
    //     let mut dat = [0u64].repeat(q.len());
    // //     let mut i = 0;
    // //     println!("{:?}", q);
    // //     for z in q {
    // //         dat[i] = memory::translate(VirtAddr::new(z.as_ref() as *const EDRPType as u64))
    // //             .unwrap()
    // //             .as_u64();
    // //         println!(" + {:#x?}", dat[i]);
    // //         i += 1;
    // //     }
    //     let val = unsafe { &mut EDRP };
    //     val.addr = memory::translate(VirtAddr::new(dat.as_ptr() as u64))
    //         .unwrap()
    //         .as_u64();
    //     val.len_or_count = dat.len() as u64;
    //     SickCustomDev::edrp_do_vectored();
    //     return mem;
    // }
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
