use crate::prelude::{DecodePacketField, EncodePacketField};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
#[error("failed to decode UUID")]
pub struct DecodeUuidError;

impl DecodePacketField for Uuid {
    type Error = DecodeUuidError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let uuid_bytes = bytes.get(*index..*index + 16).ok_or(DecodeUuidError)?;
        let uuid = Uuid::from_slice(uuid_bytes).map_err(|_| DecodeUuidError)?;
        *index += 16;
        Ok(uuid)
    }
}

impl EncodePacketField for Uuid {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>, _protocol_version: u32) -> Result<(), Self::Error> {
        bytes.extend_from_slice(self.as_bytes());
        Ok(())
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
        let mut index = 0;
        let decoded_uuid = Uuid::decode(bytes, &mut index).unwrap();

        // Then
        assert_eq!(decoded_uuid, expected_uuid);
        assert_eq!(index, 16);
    }

    #[test]
    fn test_decode_uuid_insufficient_bytes() {
        // Given
        let bytes: &[u8] = &[0; 15];
        let mut index = 0;

        // When
        let result = Uuid::decode(bytes, &mut index);

        // Then
        assert!(result.is_err());
    }
}
