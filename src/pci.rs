use crate::{dbg, print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use x86_64::instructions::port::Port;
pub const CONFIG_ADDRESS: u16 = 0xCF8;
pub const CONFIG_DATA: u16 = 0xCFC;
// pub const PCI_TABLE: &str = include_str!("../pci.txt");
// Ported from C, original at https://wiki.osdev.org/PCI
fn pci_read16(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let mut addrp: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut datap: Port<u16> = Port::new(CONFIG_DATA);
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;

    /* create configuration address as per Figure 1 */
    let address = (lbus << 16) | (lslot << 11) | (lfunc << 8) | (offset as u32) | (0x80000000);

    /* write out the address */
    unsafe {
        addrp.write(address);
    }
    /* read in the data */
    unsafe { datap.read() }
}
fn pci_read32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    (pci_read16(bus, slot, func, offset) as u32)
        | ((pci_read16(bus, slot, func, offset + 2) as u32) << 16)
}
fn pci_read8(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    pci_read16(bus, slot, func, offset) as u8
}
fn to_int(c: &str) -> Option<u16> {
    let mut r: u16 = 0;
    for ccx in c.chars() {
        r = r * 16;
        let mut cc = ccx as u8;
        if cc < 48 || cc >= 58 {
            if cc > 97 && cc < 103 {
                cc -= 97;
                cc += 48;
                cc += 10;
            } else {
                return None;
            }
        }
        r += (cc - 48) as u16;
    }
    return Some(r);
}
pub fn idlookup(vendor: u16, dev: u16) -> Option<String> {
    // let vec = PCI_TABLE.split('\n');
    // let mut iscv = false;
    // for x in vec {
    //     if x.len() < 4 {
    //         continue;
    //     }
    //     let lelr = to_int(x.clone().split_at(4).0);
    //     if lelr == Some(vendor) {
    //         println!("Found co. for {}", dev);
    //         iscv = true;
    //     } else if lelr.is_some() {
    //         if iscv {
    //             return None;
    //         }
    //         iscv = false;
    //     } else if iscv {
    //         if x.len() < 8 {
    //             continue;
    //         }
    //         let lelr2 = to_int(x.clone().split_at(5).0.split_at(1).1);
    //         if lelr2.is_some() && lelr2 == Some(dev) {
    //             return Some(x.clone().split_at(7).1.to_string());
    //         }
    //     }
    // }
    return None;
}

pub fn testing() {
    println!("Enumerating PCI");
    for bus_id in 0..255u8 {
        for slot in 0..32u8 {
            let vendor = pci_read16(bus_id, slot, 0, 0);
            let devid = pci_read16(bus_id, slot, 0, 2);
            if vendor != 0xffff && vendor != 0 {
                println!("{}:{}  {:#04x}:{:#04x}", bus_id, slot, vendor, devid);
                if let Some(x) = idlookup(vendor, devid) {
                    println!("  name = {}", x);
                }
                // let some_ram = crate::memory::alloc::ALLOCATOR.get().allocate_first_fit(alloc::alloc::Layout::from_size_align(32, 8).unwrap()).unwrap();
                println!("  header type = {}", pci_read8(bus_id, slot, 0, 0xD));
                println!("  subsystem = {}", pci_read16(bus_id, slot, 0, 0x2e));
                println!("  subsystem vendor = {}", pci_read16(bus_id, slot, 0, 0x2c));
                for i in 0..6 {
                    let barx = pci_read32(bus_id, slot, 0, 0x10 + (i * 4));
                    if barx & 1 == 1 {
                        // iospace
                        println!("  bar{} = io:{:#06x}", i, barx & 0xffffffc);
                    } else {
                        // memory
                        let bartype = (barx >> 1) & 3;
                        println!("  bar{}.raw = {:#06x}", i, barx);
                        if bartype == 0 {
                            println!("  bar{} = mem32:{:#06x}", i, barx & 0xFFFFFFF0);
                        }
                        if bartype == 2 {
                            println!(
                                "  bar{} = mem64:{:#06x}",
                                i,
                                (((barx as u64) & 0xFFFFFFF0u64)
                                    + (((pci_read32(bus_id, slot, 0, 14 + (i * 4)) & 0xFFFFFFFF)
                                        as u64)
                                        << 32))
                            );
                        }
                    }
                }
            }
        }
    }
}
