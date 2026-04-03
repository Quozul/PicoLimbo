use minecraft_protocol::prelude::*;

#[derive(PacketOut, Debug)]
pub struct SetPlayerInventoryPacket {
    slot: VarInt,
    slot_data: SlotData,
}

#[derive(Debug)]
pub enum SlotData {
    Empty,
    Item {
        item_count: VarInt,
        item_id: VarInt,
        components_to_add: Vec<VarInt>,
        components_to_remove: Vec<VarInt>,
    },
}

impl SetPlayerInventoryPacket {
    pub fn new(slot: i32, slot_data: SlotData) -> Self {
        Self {
            slot: VarInt::new(slot),
            slot_data,
        }
    }

    pub fn empty() -> Self {
        Self {
            slot: VarInt::new(0),
            slot_data: SlotData::Empty,
        }
    }

    pub fn set_slot(&mut self, slot: i32) {
        self.slot = VarInt::new(slot);
    }

    pub fn set_slot_data(&mut self, slot_data: SlotData) {
        self.slot_data = slot_data;
    }

    pub fn set_slot_data_from_id(&mut self, item_id: i32, item_count: i32) {
        self.slot_data = SlotData::Item {
            item_count: VarInt::new(item_count),
            item_id: VarInt::new(item_id),
            components_to_add: vec![],
            components_to_remove: vec![],
        };
    }
}

impl EncodePacket for SlotData {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            SlotData::Empty => {
                VarInt::new(0).encode(writer, protocol_version)?;
            }
            SlotData::Item {
                item_id,
                item_count,
                components_to_add,
                components_to_remove,
            } => {
                item_count.encode(writer, protocol_version)?;
                item_id.encode(writer, protocol_version)?;

                VarInt::new(components_to_add.len() as i32).encode(writer, protocol_version)?;

                VarInt::new(components_to_remove.len() as i32).encode(writer, protocol_version)?;

                Vec::encode(components_to_add, writer, protocol_version)?;

                Vec::encode(components_to_remove, writer, protocol_version)?;
            }
        }
        Ok(())
    }
}
