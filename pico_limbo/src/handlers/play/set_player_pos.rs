use minecraft_packets::play::set_player_position::SetPlayerPositionPacket;
use crate::handlers::play::set_player_position_and_rotation::teleport_player_to_spawn;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server_state::ServerState;

impl PacketHandler for SetPlayerPositionPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState
    ) -> Result<(), PacketHandlerError> {
        let min_y_pos_config = server_state.min_y_pos();
        if self.feet_y < f64::from(min_y_pos_config) {
            teleport_player_to_spawn(client_state, server_state);
        }

        Ok(())
    }
}