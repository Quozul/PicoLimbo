use crate::binary_reader::BinaryReader;
use crate::nbt::Nbt;
use crate::prelude::parse_tag;

impl Nbt {
    pub fn from_bytes(bytes: &[u8]) -> Nbt {
        let mut reader = BinaryReader::new(bytes);
        parse_tag(&mut reader)
    }
}
