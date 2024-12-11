pub trait SerializePacketData: Sized {
    type Error: std::error::Error;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error>;
}

macro_rules! impl_serialize_packet_data {
    ($($t:ty),*) => {
        $(
            impl SerializePacketData for $t {
                type Error = std::convert::Infallible;

                fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
                    bytes.extend_from_slice(&self.to_be_bytes());
                    Ok(())
                }
            }
        )*
    };
}

impl_serialize_packet_data!(i64, i32, f32, f64, i8, u8);

impl SerializePacketData for bool {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        bytes.push(if *self { 0x01 } else { 0x00 });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::SerializePacketData;

    #[test]
    fn test_encode_i64() {
        let mut bytes = Vec::new();
        123456789i64.encode(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 0, 7, 91, 205, 21]);
    }
}
