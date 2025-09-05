use minecraft_packets::play::set_player_position_and_rotation::SetPlayerPositionAndRotationPacket;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server_state::ServerState;

impl PacketHandler for SetPlayerPositionAndRotationPacket {
    fn handle(
        &self,
        _client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        println!(
            "SetPlayerPositionAndRotationPacket received: x={}, y={}, z={}, yaw={}, pitch={}",
            self.x, self.feet_y, self.z, self.yaw, self.pitch
        );

        Ok(())
    }
}