use crate::prelude::{DecodePacket, EncodePacket};
use crate::protocol_version::ProtocolVersion;
use pico_binutils::prelude::{BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError};

impl DecodePacket for bool {
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let value = reader.read::<u8>()?;
        Ok(value == 0x01)
    }
}

impl EncodePacket for bool {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if *self {
            writer.write::<u8>(&0x01_u8)?;
        } else {
            writer.write::<u8>(&0x00_u8)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_false_bool() {
        // Given
        let expected_value: bool = false;
        let bytes = &[0];

        // When
        let mut reader = BinaryReader::new(bytes);
        let decoded_value = bool::decode(&mut reader, ProtocolVersion::Any).unwrap();

        // Then
        assert_eq!(decoded_value, expected_value);
    }

    #[test]
    fn test_decode_true_bool() {
        // Given
        let expected_value: bool = true;
        let bytes = &[1];

        // When
        let mut reader = BinaryReader::new(bytes);
        let decoded_value = bool::decode(&mut reader, ProtocolVersion::Any).unwrap();

        // Then
        assert_eq!(decoded_value, expected_value);
    }

    #[test]
    fn test_decode_bool_insufficient_bytes() {
        // Given
        let bytes: &[u8] = &[];

        // When
        let mut reader = BinaryReader::new(bytes);
        let result = bool::decode(&mut reader, ProtocolVersion::Any);

        // Then
        assert!(result.is_err());
    }
}
