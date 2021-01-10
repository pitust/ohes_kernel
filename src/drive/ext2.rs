use crate::prelude::*;

use bitflags::bitflags;
use drive::RODev;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BlkGrpDesc {
    blkumap: u32,
    inoumap: u32,
    inotbl: u32,
}

pub struct SuperBlock {
    pub total_number_of_inodes_in_file_system: u32,
    pub total_number_of_blocks_in_file_system: u32,
    pub number_of_blocks_reserved_for_superuser: u32,
    pub total_number_of_unallocated_blocks: u32,
    pub total_number_of_unallocated_inodes: u32,
    pub block_number_of_the_block_containing_the_superblock: u32,
    pub log2_blocksize: u32,
    pub log2_fragment_size: u32,
    pub block_per_group: u32,
    pub number_of_fragments_in_each_block_group: u32,
    pub inode_per_group: u32,
    pub last_mount_time: u32,
    pub last_written_time: u32,
    pub number_of_times_the_volume_has_been_mounted_since_its_last_consistency_check: u16,
    pub number_of_mounts_allowed_before_a_consistency_check_must_be_done: u16,
    pub ext2_signature: u16,
    pub file_system_state: u16,
    pub what_to_do_when_an_error_is_detected: u16,
    pub minor_portion_of_version: u16,
    pub posix_time_of_last_consistency_check: u32,
    pub interval: u32,
    pub operating_system_id_from_which_the_filesystem_on_this_volume_was_created: u32,
    pub major_portion_of_version: u32,
    pub user_id_that_can_use_reserved_blocks: u16,
    pub group_id_that_can_use_reserved_blocks: u16,
    pub first_non_reserved: u32,
    pub inode_sz: u16,
}

pub fn handle_super_block(data: &[u8]) -> SuperBlock {
    let total_number_of_inodes_in_file_system =
        unsafe { *(data.split_at(0).1.split_at(4).0.as_ptr() as *const u32) };
    let total_number_of_blocks_in_file_system =
        unsafe { *(data.split_at(4).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_blocks_reserved_for_superuser =
        unsafe { *(data.split_at(8).1.split_at(4).0.as_ptr() as *const u32) };
    let total_number_of_unallocated_blocks =
        unsafe { *(data.split_at(12).1.split_at(4).0.as_ptr() as *const u32) };
    let total_number_of_unallocated_inodes =
        unsafe { *(data.split_at(16).1.split_at(4).0.as_ptr() as *const u32) };
    let block_number_of_the_block_containing_the_superblock =
        unsafe { *(data.split_at(20).1.split_at(4).0.as_ptr() as *const u32) };
    let log2_blocksize = unsafe { *(data.split_at(24).1.split_at(4).0.as_ptr() as *const u32) };
    let log2_fragment_size = unsafe { *(data.split_at(28).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_blocks_in_each_block_group =
        unsafe { *(data.split_at(32).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_fragments_in_each_block_group =
        unsafe { *(data.split_at(36).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_inodes_in_each_block_group =
        unsafe { *(data.split_at(40).1.split_at(4).0.as_ptr() as *const u32) };
    let last_mount_time = unsafe { *(data.split_at(44).1.split_at(4).0.as_ptr() as *const u32) };
    let last_written_time = unsafe { *(data.split_at(48).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_times_the_volume_has_been_mounted_since_its_last_consistency_check =
        unsafe { *(data.split_at(52).1.split_at(2).0.as_ptr() as *const u16) };
    let number_of_mounts_allowed_before_a_consistency_check_must_be_done =
        unsafe { *(data.split_at(54).1.split_at(2).0.as_ptr() as *const u16) };
    let ext2_signature = unsafe { *(data.split_at(56).1.split_at(2).0.as_ptr() as *const u16) };
    let file_system_state = unsafe { *(data.split_at(58).1.split_at(2).0.as_ptr() as *const u16) };
    let what_to_do_when_an_error_is_detected =
        unsafe { *(data.split_at(60).1.split_at(2).0.as_ptr() as *const u16) };
    let minor_portion_of_version =
        unsafe { *(data.split_at(62).1.split_at(2).0.as_ptr() as *const u16) };
    let posix_time_of_last_consistency_check =
        unsafe { *(data.split_at(64).1.split_at(4).0.as_ptr() as *const u32) };
    let interval = unsafe { *(data.split_at(68).1.split_at(4).0.as_ptr() as *const u32) };
    let operating_system_id_from_which_the_filesystem_on_this_volume_was_created =
        unsafe { *(data.split_at(72).1.split_at(4).0.as_ptr() as *const u32) };
    let major_portion_of_version =
        unsafe { *(data.split_at(76).1.split_at(4).0.as_ptr() as *const u32) };
    let user_id_that_can_use_reserved_blocks =
        unsafe { *(data.split_at(80).1.split_at(2).0.as_ptr() as *const u16) };
    let group_id_that_can_use_reserved_blocks =
        unsafe { *(data.split_at(82).1.split_at(2).0.as_ptr() as *const u16) };
    let first_non_reserved = unsafe { *(data.split_at(84).1.split_at(2).0.as_ptr() as *const u32) };
    let inode_sz = unsafe { *(data.split_at(88).1.split_at(2).0.as_ptr() as *const u16) };

    let sup = SuperBlock {
        total_number_of_inodes_in_file_system,
        total_number_of_blocks_in_file_system,
        number_of_blocks_reserved_for_superuser,
        total_number_of_unallocated_blocks,
        total_number_of_unallocated_inodes,
        block_number_of_the_block_containing_the_superblock,
        log2_blocksize,
        log2_fragment_size,
        block_per_group: number_of_blocks_in_each_block_group,
        number_of_fragments_in_each_block_group,
        inode_per_group: number_of_inodes_in_each_block_group,
        last_mount_time,
        last_written_time,
        number_of_times_the_volume_has_been_mounted_since_its_last_consistency_check,
        number_of_mounts_allowed_before_a_consistency_check_must_be_done,
        ext2_signature,
        file_system_state,
        what_to_do_when_an_error_is_detected,
        minor_portion_of_version,
        posix_time_of_last_consistency_check,
        interval,
        operating_system_id_from_which_the_filesystem_on_this_volume_was_created,
        major_portion_of_version,
        user_id_that_can_use_reserved_blocks,
        group_id_that_can_use_reserved_blocks,
        first_non_reserved,
        inode_sz,
    };
    // dbg!(total_number_of_inodes_in_file_system);
    // dbg!(total_number_of_blocks_in_file_system);
    // dbg!(number_of_blocks_reserved_for_superuser);
    // dbg!(total_number_of_unallocated_blocks);
    // dbg!(total_number_of_unallocated_inodes);
    // dbg!(block_number_of_the_block_containing_the_superblock);
    // dbg!(log2_blocksize);
    // dbg!(log2_fragment_size);
    // dbg!(number_of_blocks_in_each_block_group);
    // dbg!(number_of_fragments_in_each_block_group);
    // dbg!(number_of_inodes_in_each_block_group);
    // dbg!(last_mount_time);
    // dbg!(last_written_time);
    // dbg!(number_of_times_the_volume_has_been_mounted_since_its_last_consistency_check);
    // dbg!(number_of_mounts_allowed_before_a_consistency_check_must_be_done);
    if ext2_signature != 0xef53 {
        panic!("Ext2 mount failed: not ext2");
    }
    // dbg!(file_system_state);
    // dbg!(what_to_do_when_an_error_is_detected);
    // dbg!(minor_portion_of_version);
    // dbg!(posix_time_of_last_consistency_check);
    // dbg!(interval);
    // dbg!(operating_system_id_from_which_the_filesystem_on_this_volume_was_created);
    if major_portion_of_version < 1 {
        panic!(
            "Ext2 mount failed: wrong major {}",
            major_portion_of_version
        );
    }
    // dbg!(user_id_that_can_use_reserved_blocks);
    // dbg!(group_id_that_can_use_reserved_blocks);
    return sup;
}
bitflags! {
    pub struct Ext2InodeAttr: u16 {
        const OTHER_EXECUTE = 0x001;
        const OTHER_WRITE = 0x002;
        const OTHER_READ = 0x004;
        const GROUP_EXECUTE = 0x008;
        const GROUP_WRITE = 0x010;
        const GROUP_READ = 0x020;
        const USER_EXECUTE = 0x040;
        const USER_WRITE = 0x080;
        const USER_READ = 0x100;
        const STICKY = 0x200;
        const SETGID = 0x400;
        const SETUID = 0x800;

        const FIFO = 0x1000;
        const CHARACTERDEV = 0x2000;
        const DIRECTORY = 0x4000;
        const BLOCKDEV = 0x6000;
        const REGULARFILE = 0x8000;
        const SYMLINK = 0xA000;
        const SOCKET = 0xC000;

    }
}
impl Ext2InodeAttr {
    pub fn to_str(&self) -> String {
        let mut s = "".to_string();
        if self.contains(Ext2InodeAttr::DIRECTORY) {
            s += "d";
        } else if self.contains(Ext2InodeAttr::SYMLINK) {
            s += "l";
        } else if self.contains(Ext2InodeAttr::CHARACTERDEV) {
            s += "c";
        } else if self.contains(Ext2InodeAttr::BLOCKDEV) {
            s += "b";
        } else if self.contains(Ext2InodeAttr::REGULARFILE) {
            s += "-";
        } else {
            s += "?";
        }
        if self.contains(Ext2InodeAttr::USER_READ) {
            s += "r";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::USER_WRITE) {
            s += "w";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::USER_EXECUTE) {
            s += "x";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::GROUP_READ) {
            s += "r";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::GROUP_WRITE) {
            s += "w";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::GROUP_EXECUTE) {
            s += "x";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::OTHER_READ) {
            s += "r";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::OTHER_WRITE) {
            s += "w";
        } else {
            s += "-";
        }
        if self.contains(Ext2InodeAttr::OTHER_EXECUTE) {
            s += "x";
        } else {
            s += "-";
        }

        s
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Inode {
    perms: u16,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    hlinkc: u16,
    seccused: u32,
    flags: u32,
    sysspec: u32,
    blkptr0: u32,
    blkptr1: u32,
    blkptr2: u32,
    blkptr3: u32,
    blkptr4: u32,
    blkptr5: u32,
    blkptr6: u32,
    blkptr7: u32,
    blkptr8: u32,
    blkptr9: u32,
    blkptr10: u32,
    blkptr11: u32,
    singlblkptr: u32,
    dblblkptr: u32,
    triplblkptr: u32,
}

pub fn readarea(addr: u64, len: u64, dev: &mut Box<dyn RODev>) -> Vec<u8> {
    let lba = (addr / 512) as u32;
    let extraoff = addr - ((lba as u64) * 512);
    let totallen = (len + extraoff + 511) / 512;
    let mut d: Vec<u8> = (0..totallen)
        .flat_map(|f| dev.read_from((f as u32) + lba).unwrap())
        .collect();
    let mut n = d.split_off(extraoff as usize);
    drop(n.split_off(len as usize));
    drop(d);
    n
}

pub fn get_blk_grp_desc(blkgrp: u64, dev: &mut Box<dyn RODev>) -> BlkGrpDesc {
    // BlkGrpDesc
    // let addr = (0x800 + 32 /);
    let a = readarea(0x800 + (blkgrp * 0x20), 32, dev);
    let cln = unsafe { *(a.as_ptr() as *const BlkGrpDesc) }.clone();
    drop(a);
    cln
}
fn read_blk_tbl(dev: &mut Box<dyn RODev>, sb: &SuperBlock, dat: Vec<u8>) -> Vec<u8> {
    let slc = unsafe { core::slice::from_raw_parts(dat.as_ptr() as *const u32, dat.len() >> 2) };
    let mut d: Vec<(u64, u64)> = slc
        .iter()
        .filter(|a| **a != 0)
        .map(|f| {
            (
                (f << (10 + sb.log2_blocksize)) as u64,
                1 << (10 + sb.log2_blocksize) as u64,
            )
        })
        .collect();
    let t = dev.vector_read_ranges(&mut d);
    // println!("{:?}", t);
    t
}
fn read_from_inode(ino: Inode, dev: &mut Box<dyn RODev>, sb: &SuperBlock) -> Vec<u8> {
    if ino.triplblkptr != 0 {
        let dat = readarea(
            (ino.triplblkptr as u64) << (10 + sb.log2_blocksize),
            1 << (10 + sb.log2_blocksize),
            dev,
        );
        let dat = read_blk_tbl(dev, sb, dat);
        let dat = read_blk_tbl(dev, sb, dat);
        return read_blk_tbl(dev, sb, dat);
    }
    if ino.dblblkptr != 0 {
        let dat = readarea(
            (ino.dblblkptr as u64) << (10 + sb.log2_blocksize),
            1 << (10 + sb.log2_blocksize),
            dev,
        );
        let dat = read_blk_tbl(dev, sb, dat);
        return read_blk_tbl(dev, sb, dat);
    }
    if ino.singlblkptr != 0 {
        let dat = readarea(
            (ino.singlblkptr as u64) << (10 + sb.log2_blocksize),
            1 << (10 + sb.log2_blocksize),
            dev,
        );
        println!("{:?}", dat);
        return read_blk_tbl(dev, sb, dat);
    }
    [
        ino.blkptr0 as u64,
        ino.blkptr1 as u64,
        ino.blkptr2 as u64,
        ino.blkptr3 as u64,
        ino.blkptr4 as u64,
        ino.blkptr5 as u64,
        ino.blkptr6 as u64,
        ino.blkptr7 as u64,
        ino.blkptr8 as u64,
        ino.blkptr9 as u64,
        ino.blkptr10 as u64,
        ino.blkptr11 as u64,
    ]
    .iter()
    .flat_map(|f| {
        if *f != 0 {
            readarea(
                f << (10 + sb.log2_blocksize),
                1 << (10 + sb.log2_blocksize),
                dev,
            )
        } else {
            vec![]
        }
    })
    .collect()
}

pub fn readdir(dev: &mut Box<dyn RODev>, inode: u32, sb: &SuperBlock) -> BTreeMap<String, u32> {
    let group = inode / sb.inode_per_group;
    let desc = get_blk_grp_desc(group as u64, dev);
    let inode_table = (desc.inotbl as u64) << (sb.log2_blocksize + 10);
    let data = readarea(inode_table + (0x80 * (inode as u64 - 1)), 0x80, dev);
    let ino = unsafe { *(data.as_ptr() as *const Inode) }.clone();
    drop(data);
    let f = Ext2InodeAttr::from_bits(ino.perms).unwrap();
    assert!(f.contains(Ext2InodeAttr::DIRECTORY));
    let mut v = BTreeMap::new();
    let d = read_from_inode(ino, dev, sb);
    let mut doff = 0;
    loop {
        if doff == 1024 {
            break;
        }
        let d: Vec<u8> = d
            .split_at(doff)
            .1
            .split_at(0x108)
            .0
            .iter()
            .map(|f| *f)
            .collect();
        let ino2 = unsafe { *(d.as_ptr() as *mut u32) };
        if ino2 == 0 {
            break;
        }
        let len = unsafe { *(d.as_ptr().offset(4) as *mut u16) };
        doff += len as usize;
        let fnmlen = unsafe { *(d.as_ptr().offset(6) as *mut u8) } as usize;
        let d = String::from_utf8(
            d.split_at(8)
                .1
                .split_at(fnmlen)
                .0
                .iter()
                .map(|f| *f)
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        v.insert(d, ino2);
    }
    drop(d);
    v
}
pub fn cat(dev: &mut Box<dyn RODev>, inode: u32, sb: &SuperBlock) -> Vec<u8> {
    let group = inode / sb.inode_per_group;
    let desc = get_blk_grp_desc(group as u64, dev);
    let inode_table = (desc.inotbl as u64) << (sb.log2_blocksize + 10);
    let data = readarea(inode_table + (0x80 * (inode as u64 - 1)), 0x80, dev);
    let ino = unsafe { *(data.as_ptr() as *const Inode) }.clone();
    drop(data);

    let f = Ext2InodeAttr::from_bits(ino.perms).unwrap();
    assert!(f.contains(Ext2InodeAttr::REGULARFILE) || f.contains(Ext2InodeAttr::SYMLINK));
    let mut z = read_from_inode(ino, dev, sb);

    z.truncate(ino.size as usize);
    z
}
pub fn stat(dev: &mut Box<dyn RODev>, inode: u32, sb: &SuperBlock) -> Ext2InodeAttr {
    let group = inode / sb.inode_per_group;
    let desc = get_blk_grp_desc(group as u64, dev);
    let inode_table = (desc.inotbl as u64) << (sb.log2_blocksize + 10);
    let data = readarea(inode_table + (0x80 * (inode as u64 - 1)), 0x80, dev);
    let ino = unsafe { *(data.as_ptr() as *const Inode) }.clone();
    drop(data);
    Ext2InodeAttr::from_bits(ino.perms).unwrap()
}
pub fn tree(dev: &mut Box<dyn RODev>, inode: u32, sb: &SuperBlock, s: String) {
    let p = readdir(dev, inode, sb);
    let d: Vec<(&String, &u32)> = p.iter().collect();
    let mut c = 0;
    let dlen = d.len();
    for (nm, ino) in d {
        if nm.starts_with(".") {
            continue;
        }
        let flags = stat(dev, *ino, sb);
        if c + 1 == dlen {
            println!("{}\\- {} ({})", s.clone(), nm, flags.to_str());
        } else {
            println!("{}+- {} ({})", s.clone(), nm, flags.to_str());
        }

        if flags.contains(Ext2InodeAttr::DIRECTORY) && !nm.starts_with(".") {
            tree(dev, *ino, sb, s.clone() + " ");
        }
        c += 1;
    }
}
pub fn handle_rodev_with_ext2(mut dev: Box<dyn RODev>) {
    let b: Vec<u8> = vec![dev.read_from(2).unwrap(), dev.read_from(3).unwrap()]
        .into_iter()
        .flat_map(|f| f)
        .collect();
    let sup = handle_super_block(b.as_slice());
    tree(&mut dev, 2, &sup, " ".to_string());
}

pub fn traverse_fs_tree(dev: &mut Box<dyn RODev>, sb: &SuperBlock, path_elems: Vec<String>) -> u32 {
    let mut inode = 2;
    for e in path_elems {
        inode = *readdir(dev, inode, sb).get(&e).unwrap();
    }

    inode
}
