use crate::prelude::*;

use drive::RODev;

pub struct GPTPart {
    name: String,
    drive: Box<dyn RODev>,
    guid: String,
    slba: u32,
    sz: u32,
}
impl GPTPart {
    pub fn name(&self) -> String {
        return self.name.clone();
    }
    pub fn guid(&self) -> String {
        return self.guid.clone();
    }
    pub fn slba(&self) -> u32 {
        return self.slba;
    }
    pub fn sz(&self) -> u32 {
        return self.sz;
    }
}
impl RODev for GPTPart {
    fn read_from(&mut self, lba: u32) -> Result<Vec<u8>, String> {
        if self.sz <= lba {
            return Err("GPT fault: OOB read".to_string());
        }
        return self.drive.read_from(lba + self.slba);
    }
}
pub trait GetGPTPartitions {
    fn get_gpt_partitions(
        &mut self,
        clone_rodev: Box<dyn Fn<(), Output = Box<dyn RODev>>>,
    ) -> Vec<GPTPart>;
}
impl GetGPTPartitions for dyn RODev {
    fn get_gpt_partitions(
        &mut self,
        clone_rodev: Box<dyn Fn<(), Output = Box<dyn RODev>>>,
    ) -> Vec<GPTPart> {
        let mut p = vec![];
        let a = self.read_from(1).unwrap();
        let efi_magic = String::from_utf8(a.split_at(8).0.to_vec()).unwrap();
        if efi_magic != "EFI PART" {
            panic!("Invalid efi drive");
        }
        let startlba = unsafe { *(a.as_ptr().offset(0x48) as *mut u64) };
        let partc = unsafe { *(a.as_ptr().offset(0x50) as *mut u32) };
        let mut data = vec![];
        for i in 0..(((partc as u32) + 3) / 4) {
            let dat = self.read_from(i + (startlba as u32)).unwrap();
            for de in dat {
                data.push(de);
            }
        }
        // 0x0	16	Partition Type GUID (zero means unused entry)
        // 0x10	16	Unique Partition GUID
        // 0x20	8	StartingLBA
        // 0x28	8	EndingLBA
        // 0x30	8	Attributes
        // 0x38	72	Partition Name

        for part in 0..(partc) {
            let startoff = 0x80 * part;
            let data = data.split_at(startoff as usize).1;
            let parttype = prntguuid(data.split_at(16).0);
            let slba = unsafe { *(data.split_at(0x20).1.as_ptr() as *const u64) };
            let elba = unsafe { *(data.split_at(0x28).1.as_ptr() as *const u64) };
            let partid = prntguuid(data.split_at(16).1.split_at(16).0);
            let partname =
                String::from_utf8(data.split_at(0x38).1.split_at(72).0.to_vec()).unwrap();
            if parttype == "00000000-0000-0000-000000000000" {
                continue;
            }
            p.push(GPTPart {
                name: partname,
                guid: partid,
                drive: clone_rodev(),
                slba: slba as u32,
                sz: (elba - slba + 1) as u32,
            })
        }
        p
    }
}

pub fn test0() {
    let mut d = unsafe { drive::Drive::new(true, 0x1F0, 0x3F6) };
    let d2: &mut dyn RODev = &mut d;
    let p = d2.get_gpt_partitions(box (|| box (unsafe { drive::Drive::new(true, 0x1F0, 0x3F6) })));
    for pa in p {
        // println!("{}", pa.name());
        drive::ext2::handle_rodev_with_ext2(box pa);
    }
}
pub fn prntguuid(g: &[u8]) -> String {
    // 8 4 4 12

    let h = hex::encode(g.split_at(16).0);
    let (s1, h) = h.split_at(8);
    let (s2, h) = h.split_at(4);
    let (s3, h) = h.split_at(4);
    let (s4, _h) = h.split_at(12);
    return "".to_string() + s1 + "-" + s2 + "-" + s3 + "-" + s4;
}
