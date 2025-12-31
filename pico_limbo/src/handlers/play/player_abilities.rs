use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::server_bound_player_abilities_packet::ServerBoundPlayerAbilitiesPacket;

impl PacketHandler for ServerBoundPlayerAbilitiesPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        client_state.set_is_flying(self.is_flying());
        Ok(Batch::new())
    }
}
