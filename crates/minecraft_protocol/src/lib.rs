extern crate core;

pub mod data;
mod data_types;
mod packet_serializer;
pub mod state;

pub mod prelude {
    pub use crate::data_types::bit_set::BitSet;
    pub use crate::data_types::identifier::Identifier;
    pub use crate::data_types::optional::{Omitted, Optional};
    pub use crate::data_types::position::Position;
    pub use crate::data_types::prefixed::LengthPaddedVec;
    pub use crate::packet_serializer::decode_packet::DecodePacket;
    pub use crate::packet_serializer::encode_packet::EncodePacket;
    pub use crate::packet_serializer::packet_id::PacketId;
    pub use macros::PacketIn;
    pub use macros::PacketOut;
    pub use macros::packet_id;
    pub use pico_binutils::prelude::{
        BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarInt,
        VarIntPrefixedString,
    };
    pub use pico_nbt::prelude::*;
    pub use protocol_version::protocol_version::ProtocolVersion;
    pub use uuid::Uuid;
}
