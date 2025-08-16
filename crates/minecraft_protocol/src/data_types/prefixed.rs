use crate::prelude::{DecodePacket, EncodePacket, ProtocolVersion};
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, Prefixed, ReadLengthPrefix,
    VarIntPrefixed, WriteLengthPrefix,
};

/// A wrapper around a Vec that adds the length as a VarInt before the Vec itself.
pub type LengthPaddedVec<T> = VarIntPrefixed<Vec<T>>;

impl<L, T> DecodePacket for Prefixed<L, Vec<T>>
where
    L: ReadLengthPrefix,
    T: DecodePacket,
{
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let size = L::read_to_usize(reader)?;
        let mut vec: Vec<T> = Vec::with_capacity(size);
        for _i in 0..size {
            vec.push(T::decode(reader, protocol_version)?);
        }
        Ok(Self::new(vec))
    }
}

impl<L, T> EncodePacket for Prefixed<L, Vec<T>>
where
    L: WriteLengthPrefix,
    T: EncodePacket,
{
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let inner = self.inner();
        L::write_from_usize(writer, inner.len())?;
        for item in inner {
            item.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}
