use std::env;
use xfat::headers::fs::disk;
use xfat::headers::mbr;
use xfat::headers::reader::*;

/*
██████╗ ██╗   ██╗██████╗ ████████╗
██╔══██╗██║   ██║██╔══██╗╚══██╔══╝
██████╔╝██║   ██║██████╔╝   ██║
██╔═══╝ ██║   ██║██╔══██╗   ██║
██║     ╚██████╔╝██║  ██║   ██║
╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   Partition Unified ReadTer
*/

fn main() {
	let file_arg = env::args().nth(1).unwrap();

	// start building our disk
	let mut d: disk::Disk = disk::Disk {
		mbr: read_header_from_offset::<mbr::Mbr>(&file_arg, 0),
		pt_type: disk::PartitionTableType::Mbr,
		partitions: vec![],
		file_arg: file_arg.clone(),
	};
	d.mbr.pretty_print();

	// get that first partition to check for GPT
	d.set_partition_table_type(); // will panic on unimplemented partition type
	d.register_partitions();
	d.print_partitions_pretty();
	for part in d.partitions.clone().into_iter() {
		match part.p_type {
			disk::PartitionType::Ext4 => {
				let ext4part = part.clone();
				let mut ext4_reader = d.make_ext4_reader(ext4part);
				//if !ext4_reader.s.uses_64bit() {
				//continue;
				//}
				ext4_reader.populate_block_groups();
				ext4_reader.validate_block_groups();
				ext4_reader.populate_inodes();
			}
			disk::PartitionType::Unused => { /* */ }
			_ => {
				println!("Note: Partition type {:?} is not implemented.", part.p_type);
			}
		}
	}
}