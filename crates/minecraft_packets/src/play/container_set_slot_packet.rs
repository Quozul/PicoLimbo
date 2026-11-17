use minecraft_protocol::prelude::*;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use pico_nbt::Value;

/// Clientbound container set slot used to fill the lectern book slot.
#[derive(Clone)]
pub struct ContainerSetSlotPacket {
    pub window_id: i8,
    pub state_id: i32,
    pub slot: i16,
    pub book_item_nbt: Option<Value>,
    pub written_book_item_id: i32,
}

impl ContainerSetSlotPacket {
    pub fn lectern_book(window_id: i8, book_item_nbt: Value, written_book_item_id: i32) -> Self {
        Self {
            window_id,
            state_id: 1,
            slot: 0,
            book_item_nbt: Some(book_item_nbt),
            written_book_item_id,
        }
    }
}

impl EncodePacket for ContainerSetSlotPacket {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.window_id.encode(writer, protocol_version)?;
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_17_1) {
            VarInt::new(self.state_id).encode(writer, protocol_version)?;
        }
        self.slot.encode(writer, protocol_version)?;
        match &self.book_item_nbt {
            None => false.encode(writer, protocol_version)?,
            Some(nbt) => {
                true.encode(writer, protocol_version)?;
                // Item stack (legacy-friendly form used by many limbo stacks):
                // present, item id, count, NBT tag.
                VarInt::new(self.written_book_item_id).encode(writer, protocol_version)?;
                1u8.encode(writer, protocol_version)?;
                // Schematic lectern Book is usually a full Item stack compound.
                // Prefer nested `tag` when present; otherwise send the compound as tag body.
                let tag = nbt
                    .get_compound()
                    .and_then(|c| c.get("tag").cloned())
                    .unwrap_or_else(|| nbt.clone());
                tag.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}

/// Lectern page property (container set data).
#[derive(PacketOut, Clone)]
pub struct ContainerSetDataPacket {
    window_id: u8,
    property: i16,
    value: i16,
}

impl ContainerSetDataPacket {
    pub fn lectern_page(window_id: u8, page: i16) -> Self {
        Self {
            window_id,
            property: 0, // LecternMenu page property
            value: page,
        }
    }
}
