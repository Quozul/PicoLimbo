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

impl BossBarAction {
    pub fn type_id(&self) -> i32 {
        match self {
            BossBarAction::Add { .. } => 0,
            BossBarAction::Remove => 1,
            BossBarAction::UpdateHealth { .. } => 2,
            BossBarAction::UpdateTitle { .. } => 3,
            BossBarAction::UpdateStyle { .. } => 4,
            BossBarAction::UpdateFlags { .. } => 5,
        }
    }
}

impl BossBarPacket {
    pub fn add(
        uuid: Uuid,
        title: Component,
        health: f32,
        color: BossBarColor,
        division: BossBarDivision,
        flags: u8,
    ) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid_most_sig: uuid.as_u64_pair().0,
            uuid_least_sig: uuid.as_u64_pair().1,
            action: BossBarAction::Add {
                title,
                health,
                color,
                division,
                flags,
            },
        }
    }
    pub fn remove(uuid: Uuid) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid_most_sig: uuid.as_u64_pair().0,
            uuid_least_sig: uuid.as_u64_pair().1,
            action: BossBarAction::Remove,
        }
    }
    pub fn update_health(uuid: Uuid, health: f32) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid_most_sig: uuid.as_u64_pair().0,
            uuid_least_sig: uuid.as_u64_pair().1,
            action: BossBarAction::UpdateHealth { health },
        }
    }

    pub fn update_title(uuid: Uuid, title: Component) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid_most_sig: uuid.as_u64_pair().0,
            uuid_least_sig: uuid.as_u64_pair().1,
            action: BossBarAction::UpdateTitle { title },
        }
    }

    pub fn update_style(uuid: Uuid, color: BossBarColor, division: BossBarDivision) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid_most_sig: uuid.as_u64_pair().0,
            uuid_least_sig: uuid.as_u64_pair().1,
            action: BossBarAction::UpdateStyle { color, division },
        }
    }

    pub fn update_flags(uuid: Uuid, flags: u8) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid_most_sig: uuid.as_u64_pair().0,
            uuid_least_sig: uuid.as_u64_pair().1,
            action: BossBarAction::UpdateFlags { flags },
        }
    }
}

impl EncodePacket for BossBarAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        VarInt::new(self.type_id()).encode(writer, protocol_version)?;
        match self {
            BossBarAction::Add {
                title,
                health,
                color,
                division,
                flags,
            } => {
                title.clone().encode(writer, protocol_version)?;
                health.encode(writer, protocol_version)?;
                VarInt::new(*color as i32).encode(writer, protocol_version)?;
                VarInt::new(*division as i32).encode(writer, protocol_version)?;
                flags.encode(writer, protocol_version)?;
            }
            BossBarAction::Remove => {}
            BossBarAction::UpdateHealth { health } => {
                health.encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateTitle { title } => {
                title.clone().encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateStyle { color, division } => {
                VarInt::new(*color as i32).encode(writer, protocol_version)?;
                VarInt::new(*division as i32).encode(writer, protocol_version)?;
            }
            BossBarAction::UpdateFlags { flags } => {
                flags.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}
