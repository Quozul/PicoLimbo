use crate::data_types::number::DecodeNumberError;
use crate::prelude::EncodePacketField;
use crate::traits::decode_packet_field::DecodePacketField;

#[derive(Debug, Default)]
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

impl DecodePacketField for Position {
    type Error = DecodeNumberError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let val = i64::decode(bytes, index)?;
        let x = (val >> 38) as f64;
        let y = (val << 52 >> 52) as f64;
        let z = (val << 26 >> 38) as f64;
        Ok(Position { x, y, z })
    }
}

impl EncodePacketField for Position {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        let val = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);
        val.encode(bytes, protocol_version)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_position() {
        let position = Position::new(18357644.0, 831.0, -20882616.0);
        let mut bytes = Vec::new();
        position.encode(&mut bytes, 0).unwrap();
        let decoded_position = Position::decode(&bytes, &mut 0).unwrap();
        assert_eq!(position.x, decoded_position.x);
        assert_eq!(position.y, decoded_position.y);
        assert_eq!(position.z, decoded_position.z);
    }
}
