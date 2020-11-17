use crate::prelude::*;

use drive::RODev;

pub struct SuperBlock {
    pub total_number_of_inodes_in_file_system: u32,
    pub total_number_of_blocks_in_file_system: u32,
    pub number_of_blocks_reserved_for_superuser: u32,
    pub total_number_of_unallocated_blocks: u32,
    pub total_number_of_unallocated_inodes: u32,
    pub block_number_of_the_block_containing_the_superblock: u32,
    pub log2_blocksize: u32,
    pub log2_fragment_size: u32,
    pub number_of_blocks_in_each_block_group: u32,
    pub number_of_fragments_in_each_block_group: u32,
    pub number_of_inodes_in_each_block_group: u32,
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
}


pub fn handle_super_block(data: &[u8]) -> SuperBlock {
    let total_number_of_inodes_in_file_system = unsafe { *(data.split_at(0).1.split_at(4).0.as_ptr() as *const u32) };
    let total_number_of_blocks_in_file_system = unsafe { *(data.split_at(4).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_blocks_reserved_for_superuser  = unsafe { *(data.split_at(8).1.split_at(4).0.as_ptr() as *const u32) };
    let total_number_of_unallocated_blocks = unsafe { *(data.split_at(12).1.split_at(4).0.as_ptr() as *const u32) };
    let total_number_of_unallocated_inodes = unsafe { *(data.split_at(16).1.split_at(4).0.as_ptr() as *const u32) };
    let block_number_of_the_block_containing_the_superblock = unsafe { *(data.split_at(20).1.split_at(4).0.as_ptr() as *const u32) };
    let log2_blocksize  = unsafe { *(data.split_at(24).1.split_at(4).0.as_ptr() as *const u32) };
    let log2_fragment_size  = unsafe { *(data.split_at(28).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_blocks_in_each_block_group = unsafe { *(data.split_at(32).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_fragments_in_each_block_group = unsafe { *(data.split_at(36).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_inodes_in_each_block_group = unsafe { *(data.split_at(40).1.split_at(4).0.as_ptr() as *const u32) };
    let last_mount_time  = unsafe { *(data.split_at(44).1.split_at(4).0.as_ptr() as *const u32) };
    let last_written_time  = unsafe { *(data.split_at(48).1.split_at(4).0.as_ptr() as *const u32) };
    let number_of_times_the_volume_has_been_mounted_since_its_last_consistency_check  = unsafe { *(data.split_at(52).1.split_at(2).0.as_ptr() as *const u16) };
    let number_of_mounts_allowed_before_a_consistency_check_must_be_done = unsafe { *(data.split_at(54).1.split_at(2).0.as_ptr() as *const u16) };
    let ext2_signature = unsafe { *(data.split_at(56).1.split_at(2).0.as_ptr() as *const u16) };
    let file_system_state  = unsafe { *(data.split_at(58).1.split_at(2).0.as_ptr() as *const u16) };
    let what_to_do_when_an_error_is_detected  = unsafe { *(data.split_at(60).1.split_at(2).0.as_ptr() as *const u16) };
    let minor_portion_of_version  = unsafe { *(data.split_at(62).1.split_at(2).0.as_ptr() as *const u16) };
    let posix_time_of_last_consistency_check  = unsafe { *(data.split_at(64).1.split_at(4).0.as_ptr() as *const u32) };
    let interval  = unsafe { *(data.split_at(68).1.split_at(4).0.as_ptr() as *const u32) };
    let operating_system_id_from_which_the_filesystem_on_this_volume_was_created  = unsafe { *(data.split_at(72).1.split_at(4).0.as_ptr() as *const u32) };
    let major_portion_of_version  = unsafe { *(data.split_at(76).1.split_at(4).0.as_ptr() as *const u32) };
    let user_id_that_can_use_reserved_blocks = unsafe { *(data.split_at(80).1.split_at(2).0.as_ptr() as *const u16) };
    let group_id_that_can_use_reserved_blocks = unsafe { *(data.split_at(82).1.split_at(2).0.as_ptr() as *const u16) };
    let sup = SuperBlock {
        total_number_of_inodes_in_file_system,
        total_number_of_blocks_in_file_system,
        number_of_blocks_reserved_for_superuser,
        total_number_of_unallocated_blocks,
        total_number_of_unallocated_inodes,
        block_number_of_the_block_containing_the_superblock,
        log2_blocksize,
        log2_fragment_size,
        number_of_blocks_in_each_block_group,
        number_of_fragments_in_each_block_group,
        number_of_inodes_in_each_block_group,
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
        group_id_that_can_use_reserved_blocks
    };
    dbg!(total_number_of_inodes_in_file_system);
    dbg!(total_number_of_blocks_in_file_system);
    dbg!(number_of_blocks_reserved_for_superuser);
    dbg!(total_number_of_unallocated_blocks);
    dbg!(total_number_of_unallocated_inodes);
    dbg!(block_number_of_the_block_containing_the_superblock);
    dbg!(log2_blocksize);
    dbg!(log2_fragment_size);
    dbg!(number_of_blocks_in_each_block_group);
    dbg!(number_of_fragments_in_each_block_group);
    dbg!(number_of_inodes_in_each_block_group);
    dbg!(last_mount_time);
    dbg!(last_written_time);
    dbg!(number_of_times_the_volume_has_been_mounted_since_its_last_consistency_check);
    dbg!(number_of_mounts_allowed_before_a_consistency_check_must_be_done);
    dbg!(ext2_signature);
    dbg!(file_system_state);
    dbg!(what_to_do_when_an_error_is_detected);
    dbg!(minor_portion_of_version);
    dbg!(posix_time_of_last_consistency_check);
    dbg!(interval);
    dbg!(operating_system_id_from_which_the_filesystem_on_this_volume_was_created);
    dbg!(major_portion_of_version);
    dbg!(user_id_that_can_use_reserved_blocks);
    dbg!(group_id_that_can_use_reserved_blocks);
    return sup;
}

pub fn handle_rodev_with_ext2(mut dev: Box<dyn RODev>) {
    let b: Vec<u8> = vec![dev.read_from(0).unwrap(), dev.read_from(1).unwrap()].into_iter().flat_map(|f| f).collect();
    let sup = handle_super_block(b.as_slice());
}