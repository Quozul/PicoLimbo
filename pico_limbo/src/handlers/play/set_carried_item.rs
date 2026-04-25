use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::set_carried_item_packet::SetCarriedItemPacket;

impl PacketHandler for SetCarriedItemPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        if !(0..=8).contains(&self.slot()) {
            return Err(PacketHandlerError::custom("Invalid hotbar slot"));
        }

        client_state.inventory().set_current_slot(self.slot());

        Ok(Batch::new())
    }
}
