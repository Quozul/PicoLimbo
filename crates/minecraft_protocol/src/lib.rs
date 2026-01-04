extern crate core;

mod data;
mod data_types;
mod packet_serializer;
mod state;

pub mod prelude {
    pub use crate::data::coordinates::Coordinates;
    pub use crate::data::dimension::Dimension;
    pub use crate::data_types::bit_set::BitSet;
    pub use crate::data_types::optional::{Omitted, Optional};
    pub use crate::data_types::position::Position;
    pub use crate::data_types::prefixed::LengthPaddedVec;
    pub use crate::data_types::uuid::{UuidAsLongs, UuidAsString};
    pub use crate::packet_serializer::decode_packet::DecodePacket;
    pub use crate::packet_serializer::encode_packet::EncodePacket;
    pub use crate::packet_serializer::packet_id::Identifiable;
    pub use crate::state::State;
    pub use macros::PacketIn;
    pub use macros::PacketOut;
    pub use pico_binutils::prelude::{
        BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarInt,
        VarIntPrefixedString,
    };
    pub use pico_identifier::Identifier;
    pub use pico_nbt::prelude::*;
    pub use protocol_version::protocol_version::ProtocolVersion;
    pub use uuid::Uuid;
}
