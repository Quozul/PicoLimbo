use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct BossBarPacket {
    #[pvn(735..)]
    pub v1_16_uuid: Uuid,
    #[pvn(..735)]
    pub uuid_most_sig: u64,
    #[pvn(..735)]
    pub uuid_least_sig: u64,
    pub action: BossBarAction,
}

pub enum BossBarAction {
    Add {
        title: Component,
        health: f32,
        color: BossBarColor,
        division: BossBarDivision,
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
        color: BossBarColor,
        division: BossBarDivision,
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

impl EncodePacket for BossBarAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            BossBarAction::Add {
                title,
                health,
                color,
                division,
                flags,
            } => {
                VarInt::new(0).encode(writer, protocol_version)?;
                title.clone().encode(writer, protocol_version)?;
                health.encode(writer, protocol_version)?;
                VarInt::new(*color as i32).encode(writer, protocol_version)?;
                VarInt::new(*division as i32).encode(writer, protocol_version)?;
                flags.encode(writer, protocol_version)?;
            }
            BossBarAction::Remove => {
                VarInt::new(1).encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateHealth { health } => {
                VarInt::new(2).encode(writer, protocol_version)?;
                health.encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateTitle { title } => {
                VarInt::new(3).encode(writer, protocol_version)?;
                title.clone().encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateStyle { color, division } => {
                VarInt::new(4).encode(writer, protocol_version)?;
                VarInt::new(*color as i32).encode(writer, protocol_version)?;
                VarInt::new(*division as i32).encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateFlags { flags } => {
                VarInt::new(5).encode(writer, protocol_version)?;
                flags.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}
