use minecraft_packets::play::set_player_position_and_rotation::SetPlayerPositionAndRotationPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;

impl PacketHandler for SetPlayerPositionAndRotationPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        let min_y_pos_config = server_state.min_y_pos();
        if self.feet_y < min_y_pos_config as f64 {
            teleport_player_to_spawn(client_state, server_state);
        }

        Ok(())
    }
}

pub fn teleport_player_to_spawn(
    client_state: &mut ClientState,
    server_state: &ServerState,
) {
    let (x, y, z) = server_state.spawn_position();
    let packet = SynchronizePlayerPositionPacket::new(x, y, z);
    client_state.queue_packet(PacketRegistry::SynchronizePlayerPosition(packet));
    if let Some(content) = server_state.min_y_message() {
        client_state.send_message(content);
    }
}