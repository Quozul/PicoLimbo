use crate::handlers::play::set_player_position_and_rotation::teleport_player_to_spawn;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::chat_command_packet::ChatCommandPacket;
use minecraft_packets::play::chat_message_packet::ChatMessagePacket;
use tracing::info;

impl PacketHandler for ChatCommandPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        run_command(client_state, server_state, self.get_command(), &mut batch);
        Ok(batch)
    }
}

impl PacketHandler for ChatMessagePacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        if let Some(command) = self.get_command()
            && command == "spawn"
        {
            run_command(client_state, server_state, command, &mut batch);
        } else {
            info!("<{}> {}", client_state.get_username(), self.get_message());
        }
        Ok(batch)
    }
}

fn run_command(
    client_state: &mut ClientState,
    server_state: &ServerState,
    command: &str,
    batch: &mut Batch<PacketRegistry>,
) {
    info!(
        "{} issued server command: /{}",
        client_state.get_username(),
        command
    );
    if command == "spawn" {
        teleport_player_to_spawn(client_state, server_state, batch);
    }
}
