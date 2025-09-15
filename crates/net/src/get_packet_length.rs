use minecraft_protocol::prelude::*;
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketLengthParseError {
    #[error("packet_in length cannot be negative")]
    NegativeLength,
    #[error("packet_in length is too large")]
    PacketTooLarge,
    #[error(transparent)]
    BinaryReader(#[from] BinaryReaderError),
    #[error("var int is too long")]
    VarIntTooLong,
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),
}

pub const MAXIMUM_PACKET_LENGTH: usize = 2_097_151;

pub fn get_packet_length(bytes: &[u8]) -> Result<usize, PacketLengthParseError> {
    let mut reader = BinaryReader::new(bytes);
    let packet_length = reader.read::<VarInt>()?.inner();

    if packet_length >= 0 {
        let packet_length = usize::try_from(packet_length)?;

        if packet_length > MAXIMUM_PACKET_LENGTH {
            Err(PacketLengthParseError::PacketTooLarge)
        } else {
            Ok(packet_length)
        }
    } else {
        Err(PacketLengthParseError::NegativeLength)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_packet_length_valid() {
        let bytes = vec![0x80, 0x01];
        let result = get_packet_length(&bytes).unwrap();
        assert_eq!(result, 128);
    }

    #[test]
    fn test_get_packet_length_invalid_var_int() {
        let bytes = vec![0xdd];
        let result = get_packet_length(&bytes);
        assert!(matches!(
            result.unwrap_err(),
            PacketLengthParseError::BinaryReader(_)
        ));
    }

    #[test]
    fn test_get_packet_length_too_large_var_int() {
        let bytes = vec![0xff, 0xff, 0xff, 0xff, 0xff];
        let result = get_packet_length(&bytes);
        assert!(matches!(
            result.unwrap_err(),
            PacketLengthParseError::BinaryReader(_)
        ));
    }

    #[test]
    fn test_get_packet_length_negative_length() {
        let bytes = vec![0xff, 0xff, 0xff, 0xff, 0x0f];
        let result = get_packet_length(&bytes);
        assert!(matches!(
            result.unwrap_err(),
            PacketLengthParseError::NegativeLength
        ));
    }

    #[test]
    fn test_get_packet_length_zero_length() {
        let bytes = vec![0x00];
        let result = get_packet_length(&bytes).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_packet_length_too_large_length() {
        let bytes = vec![0xff, 0xff, 0xff, 0xff, 0x07];
        let result = get_packet_length(&bytes);
        assert!(matches!(
            result.unwrap_err(),
            PacketLengthParseError::PacketTooLarge
        ));
    }
}
