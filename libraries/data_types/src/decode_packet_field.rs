use thiserror::Error;

pub trait DecodePacketField: Sized {
    type Error: std::error::Error;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error>;
}

#[derive(Debug, Error)]
pub enum DeserializeNumberError {
    #[error("not enough bytes")]
    InvalidData,
    #[error("not enough bytes")]
    OutOfBounds,
}

macro_rules! impl_deserialize_packet_data {
    ($($t:ty),*) => {
        $(
            impl DecodePacketField for $t {
                type Error = DeserializeNumberError;

                fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
                     if *index + std::mem::size_of::<$t>() > bytes.len() {
                         return Err(DeserializeNumberError::OutOfBounds);
                     }

                    let value = <$t>::from_be_bytes(bytes[*index..*index + std::mem::size_of::<$t>()]
                        .try_into()
                        .map_err(|_| DeserializeNumberError::InvalidData)?);
                    *index += std::mem::size_of::<$t>();
                    Ok(value)
                }
            }
        )*
    };
}

impl_deserialize_packet_data!(i64, i32, f32, f64, i8, u16, u8);

impl DecodePacketField for bool {
    type Error = std::convert::Infallible;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let value = bytes[*index] == 0x01;
        *index += 1;
        Ok(value)
    }
}
