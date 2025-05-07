extern crate core;

pub mod data;
mod data_types;
pub mod protocol_version;
pub mod state;
mod traits;

pub mod prelude {
    pub use crate::data_types::bit_set::BitSet;
    pub use crate::data_types::identifier::Identifier;
    pub use crate::data_types::length_padded_vec::{
        LengthPaddedVec, LengthPaddedVecDecodeError, LengthPaddedVecEncodeError,
    };
    pub use crate::data_types::position::Position;
    pub use crate::data_types::string::DecodeStringError;
    pub use crate::data_types::var_int::{VarInt, VarIntParseError};
    pub use crate::data_types::vec_no_length::VecEncodeError;
    pub use crate::traits::decode_packet_field::DecodePacketField;
    pub use crate::traits::encode_packet_field::EncodePacketField;
    pub use macros::PacketIn;
    pub use macros::PacketOut;
    pub use macros::Pvn;
    pub use macros::packet_id;
    pub use nbt::prelude::*;
    pub use packet_serializer::decode_packet::DecodePacket;
    pub use packet_serializer::decode_packet_error::DecodePacketError;
    pub use packet_serializer::encode_packet::EncodePacket;
    pub use packet_serializer::packet_id::PacketId;
    pub use uuid::Uuid;
}
