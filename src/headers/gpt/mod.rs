use partitions::PartitionEntry;
use serde::Deserialize;
use serde_big_array::BigArray;
pub mod partitions;
pub mod uuids;
use super::summer::*;
use crate::headers::constants::SMOL_BLOCKS;
use crate::headers::reader::*;
use crate::prettify_output;
use colored::*;
use crc::Algorithm;
use std::ops::Range;

#[derive(Deserialize, Debug)]
pub struct Gpt {
    pub signature: [u8; 8], //	Signature, can be identified by 8 bytes magic "EFI PART" (45h 46h 49h 20h 50h 41h 52h 54h)
    pub revision: [u8; 4],  //	GPT Revision
    pub size: u32,          //	Header size
    pub crc32: u32,         //	CRC32 checksum of the GPT header
    pub reserved: [u8; 4],  //	Reserved
    pub self_lba: u64,      //	The LBA containing this header
    pub alt_lba: u64,       //	The LBA of the alternate GPT header
    pub first_usable_block: u64, //	The first usable block that can be contained in a GPT entry
    pub last_usable_block: u64, //	The last usable block that can be contained in a GPT entry
    pub guid: [u8; 16],     //	GUID of the disk
    pub gpe_table_start: u64, //	Starting LBA of the GUID Partition Entry array
    pub gpe_table_entries: u32, //	Number of Partition Entries
    pub gpe_table_entry_size: u32, //	Size (in bytes) of each entry in the Partition Entry array - must be a value of 128×2ⁿ where n ≥ 0 (in the past, multiples of 8 were acceptable)
    pub gpe_table_crc32: u32,      //	CRC32 of the Partition Entry array.
    #[serde(with = "BigArray")]
    pub also_reserved: [u8; 512 - 0x5c], // Reserved (should be zeroed) 512-0x5c is 420 btw lmaoooo
}

impl Summable for Gpt {
    //offset for checksum field is ignoreable.
    fn ranges_to_zero(&self) -> Vec<std::ops::Range<usize>> {
        vec![Range {
            start: 0x10,
            end: 0x14,
        }]
    }
    fn range_to_include(&self) -> Range<usize> {
        Range {
            start: 0,
            end: 0x5c, //last reserved field is not included in sum.
        }
    }
    // GPT_CRC32 uses linux ethernet paramters with -1 init and ~ at the end
    fn data_to_include(&self) -> Vec<u8> {
        vec![]
    }
}
impl Summable32 for Gpt {
    fn crc_parameters(&self) -> &'static Algorithm<u32> {
        &Algorithm::<u32> {
            poly: 0x04c11db7,
            init: 0xffffffff,
            refin: true,
            refout: true,
            xorout: 0xFFFFFFFF,
            check: 0,
            residue: 0,
        }
    }
    fn validate_checksum(&self, sumcheck: u32) -> bool {
        self.crc32 == sumcheck
    }
}

impl Gpt {
    pub fn create_partition_table(&self, reader: &mut OnDisk) -> Vec<PartitionEntry> {
        let mut partition_table: Vec<PartitionEntry> = vec![];
        for i in 0..self.gpe_table_entries as u64 {
            let entry = reader.read_header_from_offset::<PartitionEntry>(
                self.gpe_table_start * SMOL_BLOCKS + i * self.gpe_table_entry_size as u64,
            );
            partition_table.push(entry);
        }
        partition_table
    }
    pub fn print_partition_table(&self, reader: &mut OnDisk) {
        let mut unused_counter = 0;
        prettify_output!(PartitionEntry, purple, bright_purple, {
            for i in 0..self.gpe_table_entries as u64 {
                let entry = reader.read_header_from_offset::<PartitionEntry>(
                    self.gpe_table_start * SMOL_BLOCKS + i * self.gpe_table_entry_size as u64,
                );
                if entry.is_in_use() {
                    println!(
                        "Name: {} Type: {}",
                        entry.name().bright_blue(),
                        entry.type_to_str().cyan()
                    );
                    println!("{:x?}", entry);
                } else {
                    unused_counter += 1;
                }
            }
        });

        println!(
            "{}",
            format!("skipped {} unused partition entries", unused_counter).blue()
        );
    }

    fn table_offset(&self) -> u64 {
        self.gpe_table_start * SMOL_BLOCKS
    }
    pub fn get_parition(&self, reader: &mut OnDisk, index: u32) -> PartitionEntry {
        reader.read_header_from_offset::<PartitionEntry>(
            self.table_offset() + (index * self.gpe_table_entry_size) as u64,
        )
    }

    pub fn validate_table_checksums(&self, reader: &mut OnDisk) -> bool {
        let table_crc = crate::headers::summer::crc32_bytes_from_disk(
            reader,
            self.crc_parameters(),
            self.table_offset(),
            self.gpe_table_entries as u64 * self.gpe_table_entry_size as u64,
        );
        let comparison = table_crc == self.gpe_table_crc32;
        print_valid_checksum("GPT:TABLE", comparison);
        comparison
    }
}

impl HasHeaderMagic for Gpt {
    fn magic_field_endianness(&self) -> Endianness {
        return Endianness::Big;
    }
    fn magic_field_offset(&self) -> u64 {
        0
    }
    fn magic_field_size(&self) -> u64 {
        8
    }
    //EFI PART 45h 46h 49h 20h 50h 41h 52h 54h
    fn magic_field_upcast(&self) -> u128 {
        0x4546492050415254
    }
}
