use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::pick_item_from_block_packet::PickItemFromBlockPacket;

impl PacketHandler for PickItemFromBlockPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        println!(
            "Received PickItemFromBlockPacket: location=({}, {}, {}), include_data={}",
            self.location().x(),
            self.location().y(),
            self.location().z(),
            self.include_data()
        );

        if let Some(world_context) = &server_state.world() {
            if let Some(internal_id) = world_context.get_block_at(
                self.location().x(),
                self.location().y(),
                self.location().z(),
            ) {
                println!(
                    "Block at location=({}, {}, {}) internal_id={}",
                    self.location().x(),
                    self.location().y(),
                    self.location().z(),
                    internal_id
                );
            }

            if self.include_data() {
                if let Some(block_entity) = world_context.get_block_entity_at(
                    self.location().x(),
                    self.location().y(),
                    self.location().z(),
                ) {
                    let nbt = block_entity.to_nbt(client_state.protocol_version());
                    println!(
                        "Block entity at location=({}, {}, {}) nbt={:?}",
                        self.location().x(),
                        self.location().y(),
                        self.location().z(),
                        nbt
                    );
                }
            }
        }

        Ok(Batch::new())
    }
}
