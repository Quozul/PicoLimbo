use protocol::prelude::*;
use thiserror::Error;
use tokio::io::AsyncRead;

#[derive(Error, Debug, PartialEq)]
pub enum PacketLengthParseError {
    #[error("could not deserialize_packet the var int, the length might be incomplete")]
    IncompleteLength,
    #[error("packet_in length cannot be negative")]
    NegativeLength,
    #[error("packet_in length is too large")]
    PacketTooLarge,
}

pub const MAXIMUM_PACKET_LENGTH: usize = 2_097_151;

pub async fn get_packet_length<R>(reader: &mut R) -> Result<usize, PacketLengthParseError>
where
    R: AsyncRead + Unpin + Send,
{
    let packet_length = VarInt::decode(reader)
        .await
        .map_err(|err| match err {
            VarIntParseError::VarIntTooLarge => PacketLengthParseError::PacketTooLarge,
            VarIntParseError::Io(_) => PacketLengthParseError::IncompleteLength,
        })?
        .value();

    if packet_length >= 0 {
        let packet_length = packet_length as usize;

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
        assert_eq!(
            result.unwrap_err(),
            PacketLengthParseError::IncompleteLength
        );
    }

    #[test]
    fn test_get_packet_length_too_large_var_int() {
        let bytes = vec![0xff, 0xff, 0xff, 0xff, 0xff];
        let result = get_packet_length(&bytes);
        assert_eq!(result.unwrap_err(), PacketLengthParseError::PacketTooLarge);
    }

    #[test]
    fn test_get_packet_length_negative_length() {
        let bytes = vec![0xff, 0xff, 0xff, 0xff, 0x0f];
        let result = get_packet_length(&bytes);
        assert_eq!(result.unwrap_err(), PacketLengthParseError::NegativeLength);
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
        assert_eq!(result.unwrap_err(), PacketLengthParseError::PacketTooLarge);
    }
}
