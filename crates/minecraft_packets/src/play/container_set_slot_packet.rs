use minecraft_protocol::prelude::*;
use pico_nbt::Value;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};

/// Clientbound set a single container slot.
/// Slot encoding is version-sensitive; this writes a present written_book with NBT tag when possible.
#[derive(PacketOut)]
pub struct ContainerSetSlotPacket {
    window_id: i8,
    #[protocol_version(min = V1_17_1)]
    state_id: VarInt,
    slot: i16,
    /// Encoded manually below when using `with_book_nbt`.
    item_present: bool,
}

/// Full packet built outside derive for item body.
pub struct LecternBookSlotPacket {
    pub window_id: i8,
    pub state_id: i32,
    pub slot: i16,
    pub book_item_nbt: Option<Value>,
    pub written_book_item_id: i32,
}

impl EncodePacket for LecternBookSlotPacket {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        (self.window_id as i8).encode(writer, protocol_version)?;
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_17_1) {
            VarInt::new(self.state_id).encode(writer, protocol_version)?;
        }
        self.slot.encode(writer, protocol_version)?;
        match &self.book_item_nbt {
            None => {
                false.encode(writer, protocol_version)?;
            }
            Some(nbt) => {
                true.encode(writer, protocol_version)?;
                // Pre-1.20.5-ish item: id + count + nbt. Modern clients may need components;
                // limbo still sends classic form when the schematic already holds item NBT.
                VarInt::new(self.written_book_item_id).encode(writer, protocol_version)?;
                1u8.encode(writer, protocol_version)?; // count
                // If nbt is full item stack, prefer its tag field
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
