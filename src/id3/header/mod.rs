mod synchsafe;

pub(crate) use synchsafe::{decode as decode_synchsafe, encode as encode_synchsafe};

use crate::id3::error::Id3Error;

pub(crate) const HEADER_SIZE: usize = 10;
const FILE_IDENTIFIER: &[u8; 3] = b"ID3";

#[derive(Debug, Clone, Copy)]
pub(crate) struct TagHeader {
    pub major_version: u8,
    pub unsynchronisation: bool,
    pub extended_header: bool,
    pub tag_size: u32,
}

impl TagHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, Id3Error> {
        if bytes.len() < HEADER_SIZE || &bytes[0..3] != FILE_IDENTIFIER {
            return Err(Id3Error::NoTag);
        }
        let major_version = bytes[3];
        if !matches!(major_version, 2..=4) {
            return Err(Id3Error::UnsupportedVersion(major_version));
        }
        let flags = bytes[5];
        let size_bytes: [u8; 4] = bytes[6..10].try_into().unwrap();
        Ok(Self {
            major_version,
            unsynchronisation: flags & 0x80 != 0,
            extended_header: flags & 0x40 != 0,
            tag_size: decode_synchsafe(size_bytes),
        })
    }

    pub fn write(tag_size: u32) -> [u8; HEADER_SIZE] {
        let mut out = [0u8; HEADER_SIZE];
        out[0..3].copy_from_slice(FILE_IDENTIFIER);
        out[3] = 4;
        out[4] = 0;
        out[5] = 0;
        out[6..10].copy_from_slice(&encode_synchsafe(tag_size));
        out
    }
}
