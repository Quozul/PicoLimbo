use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::pick_item_from_block_packet::PickItemFromBlockPacket;

impl PacketHandler for PickItemFromBlockPacket {
    fn handle(
        &self,
        _client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        println!(
            "Received PickItemFromBlockPacket: location=({}, {}, {}), include_data={}",
            self.location().x(),
            self.location().y(),
            self.location().z(),
            self.include_data()
        );

        Ok(Batch::new())
    }
}
