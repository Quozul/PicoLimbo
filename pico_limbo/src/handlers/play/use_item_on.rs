use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::container_set_slot_packet::{ContainerSetDataPacket, ContainerSetSlotPacket};
use minecraft_packets::play::open_screen_packet::OpenScreenPacket;
use minecraft_packets::play::use_item_on_packet::UseItemOnPacket;
use minecraft_protocol::prelude::ProtocolVersion;

/// Approximate written_book item protocol ids by version band.
fn written_book_item_id(protocol: ProtocolVersion) -> i32 {
    // From data_generator registries (moves each release).
    if protocol.is_after_inclusive(ProtocolVersion::V1_21_5) {
        1372
    } else if protocol.is_after_inclusive(ProtocolVersion::V1_21) {
        1222
    } else if protocol.is_after_inclusive(ProtocolVersion::V1_20_5) {
        1095
    } else if protocol.is_after_inclusive(ProtocolVersion::V1_20) {
        1035
    } else {
        927
    }
}

impl PacketHandler for UseItemOnPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
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

        let Some(entity) = entities.iter().find(|entity| {
            let pos = entity.get_position();
            pos.x() == x && pos.y() == y && pos.z() == z && entity.is_lectern_with_book()
        }) else {
            return Ok(batch);
        };

        let protocol = client_state.protocol_version();
        let window_id: i8 = 1;
        let item_id = written_book_item_id(protocol);

        // 1) Open lectern screen (not hand open_book).
        batch.queue(|| PacketRegistry::OpenScreen(OpenScreenPacket::lectern(window_id as i32)));

        // 2) Put the schematic book into the lectern menu slot so pages render.
        if let Some(book_nbt) = entity.lectern_book_nbt().cloned() {
            let packet = ContainerSetSlotPacket::lectern_book(window_id, book_nbt, item_id);
            batch.queue(move || PacketRegistry::ContainerSetSlot(packet));
        }

        // 3) Set current page property to 0.
        batch.queue(|| {
            PacketRegistry::ContainerSetData(ContainerSetDataPacket::lectern_page(
                window_id as u8,
                0,
            ))
        });

        Ok(batch)
    }
}
