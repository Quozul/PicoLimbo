use crate::handlers::play::set_player_position_and_rotation::teleport_player_to_spawn_out_of_bounds;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::set_player_position_packet::SetPlayerPositionPacket;

impl PacketHandler for SetPlayerPositionPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        Ok(teleport_player_to_spawn_out_of_bounds(
            client_state,
            server_state,
            self.feet_y,
        ))
    }
}
