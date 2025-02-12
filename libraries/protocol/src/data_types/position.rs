use crate::prelude::EncodePacketField;
use crate::traits::decode_packet_field::{DecodePacketField, DeserializeNumberError};
use tokio::io::AsyncRead;

#[derive(Debug)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position { x, y, z }
    }
}

#[async_trait::async_trait]
impl DecodePacketField for Position {
    type Error = DeserializeNumberError;

    async fn decode<T>(reader: &mut T) -> Result<Self, Self::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let val = i64::decode(reader).await?;
        let x = (val >> 38) as f64;
        let y = (val << 52 >> 52) as f64;
        let z = (val << 26 >> 38) as f64;
        Ok(Position { x, y, z })
    }
}

impl EncodePacketField for Position {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let val = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);
        val.encode(bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_position() {
        // Given
        let expected_position = Position::new(18357644.0, 831.0, -20882616.0);
        let mut expected_bytes = Vec::new();
        expected_position.encode(&mut expected_bytes).unwrap();
        let mut reader = tokio_test::io::Builder::new().read(&expected_bytes).build();

        // When
        let decoded_position = Position::decode(&mut reader).await.unwrap();

        // Then
        assert_eq!(expected_position.x, decoded_position.x);
        assert_eq!(expected_position.y, decoded_position.y);
        assert_eq!(expected_position.z, decoded_position.z);
    }
}
