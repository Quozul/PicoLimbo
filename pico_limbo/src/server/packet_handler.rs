use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketHandlerError {
    #[error("An error occurred while handling a packet: {0}")]
    Custom(String),
    #[error("{0}")]
    InvalidState(String),
}

impl PacketHandlerError {
    #[inline]
    pub fn custom(message: &str) -> Self {
        Self::Custom(message.to_string())
    }

    #[inline]
    pub fn invalid_state(message: &str) -> Self {
        Self::InvalidState(message.to_string())
    }
}

pub trait PacketHandler {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError>;
}
