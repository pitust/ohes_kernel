use crate::{drive::RODev, prelude::*};
use drive::gpt::GetGPTPartitions;
use serde_derive::*;
use x86_64::structures::paging::PageTableFlags;
ezy_static! { KSVC_TABLE, BTreeMap<String, Box<dyn Send + Sync + Fn<(), Output = ()>>>, BTreeMap::new() }

#[derive(Serialize, Deserialize)]
pub enum KSvcResult {
    Success,
    Failure(String),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FSOp {
    Read,
    ReadDir,
    Stat,
}
#[derive(Serialize, Deserialize)]
pub enum FSResult {
    Data(Vec<u8>),
    Dirents(Vec<String>),
    Stats(u16),
    Failure(String),
}

pub fn ksvc_init() {
    let t = KSVC_TABLE.get();

    t.insert("log".to_string(), box || {
        let d: String = postcard::from_bytes(preempt::CURRENT_TASK.box1.unwrap()).unwrap();
        print!("{}", d);
        let x = postcard::to_allocvec(&KSvcResult::Success).unwrap();
        preempt::CURRENT_TASK.get().box1 = Some(x.leak());
    });
    t.insert("pmap".to_string(), box || {
        let d: u64 = postcard::from_bytes(preempt::CURRENT_TASK.box1.unwrap()).unwrap();
        dprint!(
            "[pmap] Mapping in {:#x?} to {:#x?}",
            d,
            d + 0xffffffffc0000000
        );
        memory::map_to(
            VirtAddr::from_ptr(d as *const u8),
            VirtAddr::from_ptr((d + 0xffffffffc0000000) as *const u8),
            PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
        );
    });
    t.insert("punmap".to_string(), box || {
        let d: u64 = postcard::from_bytes(preempt::CURRENT_TASK.box1.unwrap()).unwrap();
        dprint!("[punmap] UnMapping {:#x?}", d);
        memory::munmap(VirtAddr::from_ptr(d as *const u8));
    });
    t.insert("kfs".to_string(), box || {
        let d: (FSOp, String) = postcard::from_bytes(preempt::CURRENT_TASK.box1.unwrap()).unwrap();
        let mut drv: Box<dyn RODev> = box drive::SickCustomDev {};
        let gpt = drv.get_gpt_partitions(box || box drive::SickCustomDev {});
        let tbl: Vec<Box<(dyn RODev + 'static)>> =
            gpt.into_iter().map(|x| (box x) as Box<dyn RODev>).collect();
        let mut gpt0 = tbl.into_iter().next().unwrap();

        let path_elems: Vec<String> =
            d.1.split('/')
                .map(|p| p.to_string())
                .filter(|a| a != "")
                .collect();
        let bytes_of_superblock: Vec<u8> =
            vec![gpt0.read_from(2).unwrap(), gpt0.read_from(3).unwrap()]
                .into_iter()
                .flat_map(|f| f)
                .collect();
        let sb = drive::ext2::handle_super_block(bytes_of_superblock.as_slice());
        let inode_id = drive::ext2::traverse_fs_tree(&mut gpt0, &sb, path_elems);

        let r = match d.0 {
            FSOp::Read => FSResult::Data(drive::ext2::cat(&mut gpt0, inode_id, &sb)),
            FSOp::ReadDir => FSResult::Dirents(
                drive::ext2::readdir(&mut gpt0, inode_id, &sb)
                    .into_iter()
                    .map(|k| k.0)
                    .collect(),
            ),
            FSOp::Stat => FSResult::Stats(drive::ext2::stat(&mut gpt0, inode_id, &sb).bits()),
        };
        let x = postcard::to_allocvec(&r).unwrap();
        preempt::CURRENT_TASK.get().box1 = Some(x.leak());
    });
}
pub fn dofs() {
    let d: (FSOp, String) = postcard::from_bytes(preempt::CURRENT_TASK.box1.unwrap()).unwrap();
    if d.0 == FSOp::Read {
        let r = FSResult::Data(userland::readfs(&d.1).to_vec());
        let x = postcard::to_allocvec(&r).unwrap();
        preempt::CURRENT_TASK.get().box1 = Some(x.leak());
        return
    }
    let mut drv: Box<dyn RODev> = box drive::SickCustomDev {};
    let gpt = drv.get_gpt_partitions(box || box drive::SickCustomDev {});
    let tbl: Vec<Box<(dyn RODev + 'static)>> =
        gpt.into_iter().map(|x| (box x) as Box<dyn RODev>).collect();
    let mut gpt0 = tbl.into_iter().next().unwrap();

    let path_elems: Vec<String> =
        d.1.split('/')
            .map(|p| p.to_string())
            .filter(|a| a != "")
            .collect();
    let bytes_of_superblock: Vec<u8> = vec![gpt0.read_from(2).unwrap(), gpt0.read_from(3).unwrap()]
        .into_iter()
        .flat_map(|f| f)
        .collect();
    let sb = drive::ext2::handle_super_block(bytes_of_superblock.as_slice());
    let inode_id = drive::ext2::traverse_fs_tree(&mut gpt0, &sb, path_elems);

    let r = match d.0 {
        FSOp::Read => FSResult::Data(userland::readfs(&d.1).to_vec()),
        FSOp::ReadDir => FSResult::Dirents(
            drive::ext2::readdir(&mut gpt0, inode_id, &sb)
                .into_iter()
                .map(|k| k.0)
                .collect(),
        ),
        FSOp::Stat => FSResult::Stats(drive::ext2::stat(&mut gpt0, inode_id, &sb).bits()),
    };
    let x = postcard::to_allocvec(&r).unwrap();
    preempt::CURRENT_TASK.get().box1 = Some(x.leak());
}
