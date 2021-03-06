use super::hash::hash_versions;
use colored::*;
use serde::Deserialize;

#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed)]
pub struct Root {
    dot_inode: u32,   // 	inode number of this directory.
    dot_rec_len: u16, // 	Length of this record, 12.
    dot_name_len: u8, // 	Length of the name, 1.
    dot_file_type: u8, /* 	File type of this entry, 0x2
                       * (directory) (if the feature flag
                       * is set). */
    dot_name: [u8; 4], //".\0\0\0"
    dotdot_inode: u32, /* 	inode number of parent
                        * directory. */
    dotdot_rec_len: u16, /* 	block_size - 12. The record
                          * length is long enough to
                          * cover all htree data. */
    dotdot_name_len: u8, // 	Length of the name, 2.
    dotdot_file_type: u8, /* 	File type of this entry, 0x2
                          * (directory) (if the feature
                          * flag is set). */
    dotdot_name: [u8; 4], // 	"..\0\0"
    root_info: RootInfo,
    limit: u16, /* 	Maximum number of dx_entries that can
                 * follow this header, plus 1 for the
                 * header itself. */
    pub count: u16, /* 	Actual number of dx_entries that
                     * follow this header, plus 1 for the
                     * header itself. */
    block: u32, /* The block number (within the directory file)
                 * that goes with hash=0.
                 *
                 * struct dx_entry 	entries[0] 	As many
                 * 8-byte struct dx_entry as fits in the
                 * rest of the data block. */
}

impl Root {
    pub fn hash_version(&self) -> hash_versions::HashVer {
        self.root_info.hash_version()
    }
}

macro_rules! validate_field {
    ($field:expr,$value:expr) => {
        if $field != $value {
            println!(
                "Error on field {}: {:x?} != {:x?}",
                stringify!($field),
                $field,
                $value
            );
            return false;
        }
    };
}
macro_rules! validate_field_lt {
    ($field:expr,$value:expr) => {
        if $field >= $value {
            println!("Error: {:#x?} != {:#x?}", $field, $value);
            return false;
        }
    };
}
impl Root {
    pub fn not_inode_0(&self) -> bool {
        self.dot_inode != 0
    }
    pub fn validate(&self, bs: u16) -> bool {
        let drec = self.dot_rec_len;
        validate_field!(drec, 12);
        let name_len = self.dot_name_len;
        validate_field!(name_len, 1);
        validate_field!(self.dot_file_type, 2);
        validate_field!(self.dot_name, [b'.', 0, 0, 0]);
        let ddrec = self.dotdot_rec_len;
        validate_field!(ddrec, bs - 12);
        validate_field!(self.dotdot_name_len, 2);
        validate_field!(self.dotdot_file_type, 2);
        validate_field!(self.dotdot_name, [b'.', b'.', 0, 0]);
        if !self.root_info.validate() {
            println!("root info didn't validate!");
            return false;
        }
        let possible = std::mem::size_of::<Root>() as u16
            + self.limit * std::mem::size_of::<Entry>() as u16;
        validate_field!(possible, bs);

        println!(
            "{}",
            "he'll yea brother root valleydatored".green()
        );
        true
    }
    pub fn last_level(&self) -> bool {
        self.root_info.indirect_levels == 0
    }
    pub fn indirect_levels(&self) -> usize {
        self.root_info.indirect_levels as usize
    }
}

#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed)]
pub struct RootInfo {
    reserved_zero: u32, // 	Zero.
    hash_version: u8,   // hash versions
    info_length: u8,    /* 	Length of the tree
                         * information, 0x8. */
    indirect_levels: u8, /* 	Depth of the htree. Cannot
                          * be larger than 3 if the
                          * INCOMPAT_LARGEDIR feature is
                          * set; cannot be larger than 2
                          * otherwise. */
    unused_flags: u8, //
}

impl RootInfo {
    pub fn hash_version(&self) -> hash_versions::HashVer {
        hash_versions::VAL_TO_ENUM[self.hash_version as usize].clone()
    }
}

impl RootInfo {
    pub fn validate(&self) -> bool {
        validate_field_lt!(
            self.hash_version,
            (hash_versions::SIPHASH + 1)
        );
        let reserved_zero = self.reserved_zero;
        validate_field!(reserved_zero, 0);
        validate_field!(self.info_length, 8);
        validate_field_lt!(self.indirect_levels, 4);
        //NOTE: unused flags
        true
    }
}
#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed)]
pub struct Node {
    fake_inode: u32, /* 	Zero, to make it look like this
                      * entry is not in use. */
    fake_rec_len: u16, /* 	The size of the block, in
                        * order to hide all of the
                        * dx_node data. */
    name_len: u8, /* 	Zero. There is no name for this
                   * "unused" directory entry. */
    file_type: u8, /* 	Zero. There is no file type for
                    * this "unused" directory entry. */
    limit: u16, /* 	Maximum number of dx_entries that can
                 * follow this header, plus 1 for the
                 * header itself. */
    count: u16, /* 	Actual number of dx_entries that
                 * follow this header, plus 1 for the
                 * header itself. */
    block: u32, /* The block number (within the directory file)
                 * that goes with the lowest hash value of this
                 * block. This value is stored in the parent
                 * block. */
}
#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed)]
pub struct Entry {
    pub hash: u32, // 	Hash code.
    block: u32,    /* 	Block number (within the directory
                    * file, not filesystem blocks) of the
                    * next node in the htree. */
}

impl Entry {
    pub fn get_block(&self) -> u32 {
        self.block
    }
    pub fn empty(&self) -> bool {
        self.hash == 0 && self.block == 0
    }
}

#[derive(Deserialize, Copy, Clone, Debug)]
#[repr(packed)]
pub struct Tail {
    /// FIXME: not used to validate yet
    reserved: u32,
    csum: u32, /* uuid,htree index header, all indices
                * that are in use, and tail block */
}
