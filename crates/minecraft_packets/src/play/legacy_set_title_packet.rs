use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct LegacySetTitlePacket {
    action: LegacySetTitleAction,
}

impl LegacySetTitlePacket {
    pub fn action_bar(action_bar: &Component) -> Self {
        Self {
            action: LegacySetTitleAction::SetActionBar {
                action_bar: action_bar.clone(),
            },
        }
    }

    pub fn set_title(title: &Component) -> Self {
        Self {
            action: LegacySetTitleAction::SetTitle {
                title: title.clone(),
            },
        }
    }

    pub fn set_subtitle(subtitle: &Component) -> Self {
        Self {
            action: LegacySetTitleAction::SetSubtitle {
                subtitle: subtitle.clone(),
            },
        }
    }

    pub fn set_animation(fade_in: i32, stay: i32, fade_out: i32) -> Self {
        Self {
            action: LegacySetTitleAction::SetTimesAndDisplay {
                fade_in,
                stay,
                fade_out,
            },
        }
    }
}

#[allow(dead_code)]
enum LegacySetTitleAction {
    SetTitle {
        title: Component,
    },
    SetSubtitle {
        subtitle: Component,
    },
    SetActionBar {
        action_bar: Component,
    },
    SetTimesAndDisplay {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    Hide,
    Reset,
}

impl LegacySetTitleAction {
    fn type_id(&self, protocol_version: ProtocolVersion) -> Result<i32, BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_11) {
            let type_id = match self {
                LegacySetTitleAction::SetTitle { .. } => 0,
                LegacySetTitleAction::SetSubtitle { .. } => 1,
                LegacySetTitleAction::SetActionBar { .. } => 2,
                LegacySetTitleAction::SetTimesAndDisplay { .. } => 3,
                LegacySetTitleAction::Hide => 4,
                LegacySetTitleAction::Reset => 5,
            };
            Ok(type_id)
        } else {
            match self {
                LegacySetTitleAction::SetTitle { .. } => Ok(0),
                LegacySetTitleAction::SetSubtitle { .. } => Ok(1),
                LegacySetTitleAction::SetTimesAndDisplay { .. } => Ok(2),
                LegacySetTitleAction::Hide => Ok(3),
                LegacySetTitleAction::Reset => Ok(4),
                LegacySetTitleAction::SetActionBar { .. } => {
                    Err(BinaryWriterError::UnsupportedOperation)
                }
            }
        }
    }
}

impl EncodePacket for LegacySetTitleAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let type_id = VarInt::new(self.type_id(protocol_version)?);
        type_id.encode(writer, protocol_version)?;
        match self {
            LegacySetTitleAction::SetTitle { title } => {
                title.encode(writer, protocol_version)?;
            }
            LegacySetTitleAction::SetSubtitle { subtitle } => {
                subtitle.encode(writer, protocol_version)?;
            }
            LegacySetTitleAction::SetActionBar { action_bar } => {
                if protocol_version.is_before_inclusive(ProtocolVersion::V1_10) {
                    Err(BinaryWriterError::UnsupportedOperation)?;
                }
                action_bar.encode(writer, protocol_version)?;
            }
            LegacySetTitleAction::SetTimesAndDisplay {
                fade_in,
                stay,
                fade_out,
            } => {
                fade_in.encode(writer, protocol_version)?;
                stay.encode(writer, protocol_version)?;
                fade_out.encode(writer, protocol_version)?;
            }
            LegacySetTitleAction::Hide | LegacySetTitleAction::Reset => {
                // Nothing to encode
            }
        }
        Ok(())
    }
}
