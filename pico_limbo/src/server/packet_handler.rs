use crate::handlers::UnknownStateError;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_protocol::prelude::ProtocolVersion;
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketHandlerError {
    #[error("Unhandled packet {0}")]
    UnhandledPacket(String),
    #[error("Dimension index was not found for version {0}")]
    DimensionIndexNotFound(ProtocolVersion),
    #[error("Cannot build login packet for version {0}")]
    CannotBuildLoginPacket(ProtocolVersion),
    #[error("Player must connect through a proxy")]
    ProxyRequired,
    #[error(transparent)]
    UnknownState(#[from] UnknownStateError),
    #[error("Missing secret key")]
    MissingSecretKey,
    #[error("Configuration state not supported for version {0}")]
    ConfigurationStateNotHandled(ProtocolVersion),
    #[error("Conversion failed: Invalid or out-of-range float")]
    DoubleConversionFailed,
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),
    #[error("Cannot find void biome index for version {0}")]
    BiomeNotFound(ProtocolVersion),
}

pub trait PacketHandler {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError>;
}
