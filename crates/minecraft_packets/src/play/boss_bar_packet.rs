use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct BossBarPacket {
    uuid: UuidAsLongs,
    action: BossBarAction,
}

impl BossBarPacket {
    pub fn add(
        title: &Component,
        health: f32,
        color: BossBarColor,
        division: BossBarDivision,
    ) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            uuid: uuid.into(),
            action: BossBarAction::Add {
                title: title.clone(),
                health,
                color: VarInt::from(color as i32),
                division: VarInt::from(division as i32),
                flags: 0,
            },
        }
    }
}

#[allow(dead_code)]
enum BossBarAction {
    Add {
        title: Component,
        health: f32,
        color: VarInt,
        division: VarInt,
        /// Bit mask. 0x01: should darken sky, 0x02: is dragon bar (used to play end music), 0x04: create fog (previously was also controlled by 0x02).
        flags: u8,
    },
    Remove,
    UpdateHealth {
        health: f32,
    },
    UpdateTitle {
        title: Component,
    },
    UpdateStyle {
        color: VarInt,
        division: VarInt,
    },
    UpdateFlags {
        flags: u8,
    },
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum BossBarColor {
    Pink = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    White = 6,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum BossBarDivision {
    NoDivision = 0,
    SixNotches = 1,
    TenNotches = 2,
    TwelveNotches = 3,
    TwentyNotches = 4,
}

impl BossBarAction {
    fn type_id(&self) -> VarInt {
        match self {
            Self::Add { .. } => 0.into(),
            Self::Remove => 1.into(),
            Self::UpdateHealth { .. } => 2.into(),
            Self::UpdateTitle { .. } => 3.into(),
            Self::UpdateStyle { .. } => 4.into(),
            Self::UpdateFlags { .. } => 5.into(),
        }
    }
}

impl EncodePacket for BossBarAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.type_id().encode(writer, protocol_version)?;
        match self {
            Self::Add {
                title,
                health,
                color,
                division,
                flags,
            } => {
                title.encode(writer, protocol_version)?;
                health.encode(writer, protocol_version)?;
                color.encode(writer, protocol_version)?;
                division.encode(writer, protocol_version)?;
                flags.encode(writer, protocol_version)?;
            }
            Self::Remove => {
                // Nothing to encode
            }
            Self::UpdateHealth { health } => {
                health.encode(writer, protocol_version)?;
            }
            Self::UpdateTitle { title } => {
                title.encode(writer, protocol_version)?;
            }
            Self::UpdateStyle { color, division } => {
                color.encode(writer, protocol_version)?;
                division.encode(writer, protocol_version)?;
            }
            Self::UpdateFlags { flags } => {
                flags.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}
