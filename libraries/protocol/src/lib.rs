extern crate core;

mod data_types;
mod decode_packet_error;
mod deserialize_packet;
mod serialize_packet;
mod traits;

pub mod prelude {
    pub use crate::data_types::bit_set::BitSet;
    pub use crate::data_types::identifier::Identifier;
    pub use crate::data_types::length_padded_vec::{
        LengthPaddedVec, LengthPaddedVecDecodeError, LengthPaddedVecEncodeError,
    };
    pub use crate::data_types::nbt::NbtEncodeError;
    pub use crate::data_types::optional::DecodeOptionError;
    pub use crate::data_types::position::Position;
    pub use crate::data_types::string::StringDecodingError;
    pub use crate::data_types::var_int::{VarInt, VarIntParseError};
    pub use crate::data_types::vec_no_length::VecEncodeError;
    pub use crate::decode_packet_error::DecodePacketError;
    pub use crate::deserialize_packet::DeserializePacketData;
    pub use crate::serialize_packet::SerializePacketData;
    pub use crate::traits::decode_packet::DecodePacket;
    pub use crate::traits::encode_packet::EncodePacket;
    pub use crate::traits::packet_id::PacketId;
    pub use macros::PacketIn;
    pub use macros::PacketOut;
    pub use macros::packet_id;
    pub use nbt::prelude::*;
    pub use uuid::Uuid;
}
