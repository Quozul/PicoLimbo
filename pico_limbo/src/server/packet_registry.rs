use macros::PacketReport;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, DecodePacket, EncodePacket,
    ProtocolVersion,
};
use minecraft_protocol::state::State;

#[derive(PacketReport)]
pub enum PacketRegistry {
    #[protocol_id(
        state = "handshake",
        bound = "serverbound",
        name = "minecraft:intention"
    )]
    Handshake(HandshakePacket),

    #[protocol_id(
        state = "status",
        bound = "serverbound",
        name = "minecraft:status_request"
    )]
    StatusRequest(StatusRequestPacket),

    #[protocol_id(
        state = "status",
        bound = "clientbound",
        name = "minecraft:status_response"
    )]
    StatusResponse(StatusResponsePacket),
}
