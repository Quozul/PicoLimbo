use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;
use std::borrow::Cow;

impl<T: EncodePacket> EncodePacket for Vec<T> {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        for value in self {
            value.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}

impl<T: EncodePacket> EncodePacket for Cow<'_, [T]>
where
    [T]: ToOwned,
{
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        for value in self.iter() {
            value.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}

/// Decoding a vec u8 implies reading elements until the buffer is exhausted.
impl DecodePacket for Vec<u8> {
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let remaining_count = reader.remaining();
        let mut buffer = vec![0u8; remaining_count];
        let bytes_read = reader.read_bytes(&mut buffer)?;
        if bytes_read != remaining_count {
            Err(BinaryReaderError::UnexpectedEof)
        } else {
            Ok(buffer)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::EncodePacket;
    use pico_binutils::prelude::{BinaryWriter, VarInt};
    use protocol_version::protocol_version::ProtocolVersion;

    #[test]
    fn test_vec_encode() {
        let vec = vec![VarInt::new(1), VarInt::new(2)];
        let mut writer = BinaryWriter::new();
        vec.encode(&mut writer, ProtocolVersion::Any).unwrap();
        let bytes = writer.into_inner();
        assert_eq!(bytes, vec![0x01, 0x02]);
    }

    #[test]
    fn test_vec_encode_empty() {
        let vec = Vec::<VarInt>::new();
        let mut writer = BinaryWriter::new();
        vec.encode(&mut writer, ProtocolVersion::Any).unwrap();
        let bytes = writer.into_inner();
        assert!(bytes.is_empty());
    }
}
