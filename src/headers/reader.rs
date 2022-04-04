use bincode::deserialize;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

pub trait HasRawHeader<Header: Sized, RawHeader: Sized> {
    fn from_raw(raw: &RawHeader) -> Header;
}


pub fn read_header_from_file<
    Header: Sized + DeserializeOwned
    //HeaderRaw: Sized + DeserializeOwned,
>(
    file_arg: &str,
    offset: u64,
) -> Header {
    let header: Header = {
        let mut file = File::open(file_arg).unwrap();
        let _res = file.seek(SeekFrom::Start(offset)).unwrap();
        if _res != offset {
            panic!("Failed to seek to offset\n"); //shitty error msg i'm tired
        }
        let size = size_of::<Header>();
        let mut file_data: Vec<u8> = vec![0; size];
        file.read_exact(&mut file_data[..]).unwrap();
        // read the bytes into the struct
        deserialize::<Header>(&file_data[..]).unwrap()
    };
    header
}

pub fn le_u128_deserialize<'de, D>(d: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let mut data = <u128>::deserialize(d)?;
    data = u128::from_le(data);
    Ok(data)
}
pub fn le_u64_deserialize<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut data = <u64>::deserialize(d)?;
    data = u64::from_le(data);
    Ok(data)
}

pub fn le_u32_deserialize<'de, D>(d: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let mut data = <u32>::deserialize(d)?;
    data = u32::from_le(data);
    Ok(data)
}
pub fn le_u16_deserialize<'de, D>(d: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let mut data = <u16>::deserialize(d)?;
    data = u16::from_le(data);
    Ok(data)
}




pub fn read_header_from_file_unsafe<
    Header: Sized + HasRawHeader<Header, HeaderRaw>,
    HeaderRaw: Sized + DeserializeOwned,
>(
    file_arg: &str,
    offset: u64,
) -> Header {
    let header: HeaderRaw = {
        let mut file = File::open(file_arg).unwrap();
        let _res = file.seek(SeekFrom::Start(offset)).unwrap();
        if _res != offset {
            panic!("Failed to seek to offset\n"); //shitty error msg i'm tired
        }
        let size = size_of::<HeaderRaw>();
        let mut file_data: Vec<u8> = vec![0; size];
        file.read_exact(&mut file_data[..]).unwrap();
        // read the bytes into the struct
        deserialize::<HeaderRaw>(&file_data[..]).unwrap()
    };
    Header::from_raw(&header)
}
