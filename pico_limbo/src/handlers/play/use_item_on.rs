use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::open_screen_packet::OpenScreenPacket;
use minecraft_packets::play::use_item_on_packet::UseItemOnPacket;

impl PacketHandler for UseItemOnPacket {
    fn handle(
        &self,
        _client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch, PacketHandlerError> {
        let mut batch = Batch::new();
        let (x, y, z) = self.block_pos();

        let Some(world) = server_state.world() else {
            return Ok(batch);
        };

        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let Some(entities) = world.get_chunk_block_entities(chunk_x, chunk_z) else {
            return Ok(batch);
        };

        let has_lectern_book = entities.iter().any(|entity| {
            let pos = entity.get_position();
            pos.x() == x && pos.y() == y && pos.z() == z && entity.is_lectern_with_book()
        });

        if has_lectern_book {
            // Lectern UI — not open_book hand packet. Book stays on the lectern.
            batch.queue(|| PacketRegistry::OpenScreen(OpenScreenPacket::lectern(1)));
        }

        Ok(batch)
    }
}
