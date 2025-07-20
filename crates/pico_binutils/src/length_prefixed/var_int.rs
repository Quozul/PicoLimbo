use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_writer::{BinaryWriter, BinaryWriterError};
use crate::length_prefixed::reader::{ReadLengthPrefix, from_i32};
use crate::length_prefixed::writer::{WriteLengthPrefix, get_i32};
use crate::prelude::{Prefixed, VarInt};

/// Strings and Arrays in Network format are prefixed with their length as a VarInt
pub type VarIntPrefixed<T> = Prefixed<VarInt, T>;

pub type VarIntPrefixedString = VarIntPrefixed<String>;

impl WriteLengthPrefix for VarInt {
    fn write_from_usize(writer: &mut BinaryWriter, len: usize) -> Result<(), BinaryWriterError> {
        let len_i32 = get_i32(len)?;
        let var_int = VarInt::new(len_i32);
        writer.write(&var_int)
    }
}

impl ReadLengthPrefix for VarInt {
    fn read_to_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError> {
        let len = reader.read::<VarInt>()?.inner();
        from_i32(len)
    }
}
