use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::container_set_slot_packet::{
    ContainerSetDataPacket, ContainerSetSlotPacket,
};
use minecraft_packets::play::item_registry_ids::{
    lectern_menu_protocol_id, written_book_protocol_id,
};
use minecraft_packets::play::open_screen_packet::OpenScreenPacket;
use minecraft_packets::play::use_item_on_packet::UseItemOnPacket;

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
        let Some(menu_type) = lectern_menu_protocol_id(protocol) else {
            return Ok(batch);
        };
        let Some(item_id) = written_book_protocol_id(protocol) else {
            return Ok(batch);
        };

        let window_id: i8 = 1;

        batch.queue(move || {
            PacketRegistry::OpenScreen(OpenScreenPacket::lectern(i32::from(window_id), menu_type))
        });

        if let Some(book_nbt) = entity.lectern_book_nbt().cloned() {
            let packet = ContainerSetSlotPacket::lectern_book(window_id, book_nbt, item_id);
            batch.queue(move || PacketRegistry::ContainerSetSlot(packet));
        }

        batch.queue(move || {
            PacketRegistry::ContainerSetData(ContainerSetDataPacket::lectern_page(
                window_id as u8,
                0,
            ))
        });

        Ok(batch)
    }
}
