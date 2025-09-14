use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarIntPrefixedString,
};
use protocol_version::protocol_version::ProtocolVersion;
use uuid::Uuid;

impl DecodePacket for Uuid {
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        if protocol_version >= ProtocolVersion::V1_16 {
            reader.read::<Uuid>()
        } else {
            Err(BinaryReaderError::Custom)
        }
    }
}

/// A UUID wrapper that encodes as a string representation based on protocol version.
///
/// - Before 1.7.6: Encoded as a string without dashes (e.g., "550e8400e29b41d4a716446655440000")
/// - 1.7.6 to 1.15: Encoded as a string with dashes (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// - 1.16+: Encoded as raw 16-byte array
#[derive(Default)]
pub struct UuidAsString(Uuid);

impl UuidAsString {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<Uuid> for UuidAsString {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl EncodePacket for UuidAsString {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_16) {
            // Since 1.16 (inclusive), UUIDs are sent as bytes
            let uuid_bytes = self.0.as_bytes().as_slice();
            writer.write_bytes(uuid_bytes)?;
            Ok(())
        } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_7_6) {
            // Since 1.7.6 (inclusive), UUIDs are sent as strings separated by dashes
            let string = VarIntPrefixedString::string(self.0);
            writer.write(&string)
        } else {
            // Before 1.7.6 (exclusive), UUIDs are sent as strings without the dashes
            let string = self.0.to_string().replace("-", "");
            let protocol_string = VarIntPrefixedString::string(string);
            writer.write(&protocol_string)
        }
    }
}

/// A UUID wrapper that encodes as either raw bytes or a pair of 64-bit integers.
///
/// - Before 1.16: Encoded as two i64 values (most significant bits, least significant bits)
/// - 1.16+: Encoded as raw 16-byte array
pub struct UuidAsLongs(Uuid);

impl UuidAsLongs {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<Uuid> for UuidAsLongs {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl EncodePacket for UuidAsLongs {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_16) {
            // Since 1.16 (inclusive), UUIDs are sent as bytes
            let uuid_bytes = self.0.as_bytes().as_slice();
            writer.write_bytes(uuid_bytes)?;
            Ok(())
        } else {
            // Before 1.16, this Uuid variant is encoded as a pair of long
            let (most_sig, least_sig) = self.0.as_u64_pair();
            writer.write(&most_sig)?;
            writer.write(&least_sig)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_decode_valid_uuid() {
        // Given
        let expected_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = expected_uuid.as_bytes();

        // When
        let mut reader = BinaryReader::new(bytes);
        let decoded_uuid = Uuid::decode(&mut reader, ProtocolVersion::V1_16).unwrap();

        // Then
        assert_eq!(decoded_uuid, expected_uuid);
    }

    #[test]
    fn test_decode_uuid_insufficient_bytes() {
        // Given
        let bytes: &[u8] = &[0; 15];
        let mut reader = BinaryReader::new(bytes);

        // When
        let result = Uuid::decode(&mut reader, ProtocolVersion::Any);

        // Then
        assert!(result.is_err());
    }
}
