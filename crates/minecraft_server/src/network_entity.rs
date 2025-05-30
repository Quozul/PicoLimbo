use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use net::packet_stream::PacketStreamError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientReadPacketError {
    #[error(transparent)]
    PacketStream(#[from] PacketStreamError),
    #[error("unknown packet id=0x{id:02X} received in state {state}")]
    UnknownPacketId { id: u8, state: State },
    #[error(
        "unknown packet name '{name}' (id=0x{id:02X}) for state {state} & protocol {protocol:?}"
    )]
    UnknownPacketName {
        name: String,
        id: u8,
        state: State,
        protocol: ProtocolVersion,
    },
    #[error("empty packet data received in state {state}")]
    EmptyPacketData { state: State },
}

#[derive(Debug, Error)]
pub enum ClientSendPacketError {
    #[error(
        "packet '{packet_name}' not found in packet map for version {version:?} / state {state:?}"
    )]
    UnmappedPacket {
        packet_name: String,
        version: ProtocolVersion,
        state: State,
    },
    #[error("failed to encode packet '{packet_name}'")]
    EncodingError { packet_name: String },
    #[error(transparent)]
    PacketStream(#[from] PacketStreamError),
    #[error("client protocol version not set, cannot send packet '{packet_name}'")]
    VersionNotSet { packet_name: String },
}
