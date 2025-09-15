use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct LegacySetTitlePacket {
    #[pvn(315..)]
    action: LegacySetTitleAction,
    #[pvn(..315)]
    v1_11_action: LegacyV1_11SetTitleAction,
}

impl LegacySetTitlePacket {
    pub fn set_title(title: &Component) -> Self {
        Self {
            action: LegacySetTitleAction::SetTitle {
                title: title.clone(),
            },
            v1_11_action: LegacyV1_11SetTitleAction::SetTitle {
                title: title.clone(),
            },
        }
    }

    pub fn set_subtitle(subtitle: &Component) -> Self {
        Self {
            action: LegacySetTitleAction::SetSubtitle {
                subtitle: subtitle.clone(),
            },
            v1_11_action: LegacyV1_11SetTitleAction::SetSubtitle {
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
            v1_11_action: LegacyV1_11SetTitleAction::SetTimesAndDisplay {
                fade_in,
                stay,
                fade_out,
            },
        }
    }

    pub fn hide() -> Self {
        Self {
            action: LegacySetTitleAction::Hide {},
            v1_11_action: LegacyV1_11SetTitleAction::Hide {},
        }
    }

    pub fn reset() -> Self {
        Self {
            action: LegacySetTitleAction::Reset {},
            v1_11_action: LegacyV1_11SetTitleAction::Reset {},
        }
    }

    pub fn action_bar(action_bar: &Component) -> Self {
        Self {
            action: LegacySetTitleAction::SetActionBar {
                action_bar: action_bar.clone(),
            },
            v1_11_action: LegacyV1_11SetTitleAction::Hide {},
        }
    }

    pub fn create_title(
        title: &Component,
        subtitle: &Component,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    ) -> Vec<Self> {
        vec![
            Self::set_title(title),
            Self::set_subtitle(subtitle),
            Self::set_animation(fade_in, stay, fade_out),
        ]
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
    Hide {},
    Reset {},
}

enum LegacyV1_11SetTitleAction {
    SetTitle {
        title: Component,
    },
    SetSubtitle {
        subtitle: Component,
    },
    SetTimesAndDisplay {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    Hide {},
    Reset {},
}

impl LegacySetTitleAction {
    fn type_id(&self) -> u8 {
        match self {
            LegacySetTitleAction::SetTitle { .. } => 0,
            LegacySetTitleAction::SetSubtitle { .. } => 1,
            LegacySetTitleAction::SetActionBar { .. } => 2,
            LegacySetTitleAction::SetTimesAndDisplay { .. } => 3,
            LegacySetTitleAction::Hide {} => 4,
            LegacySetTitleAction::Reset {} => 5,
        }
    }
}

impl LegacyV1_11SetTitleAction {
    fn type_id(&self) -> u8 {
        match self {
            LegacyV1_11SetTitleAction::SetTitle { .. } => 0,
            LegacyV1_11SetTitleAction::SetSubtitle { .. } => 1,
            LegacyV1_11SetTitleAction::SetTimesAndDisplay { .. } => 2,
            LegacyV1_11SetTitleAction::Hide {} => 3,
            LegacyV1_11SetTitleAction::Reset {} => 4,
        }
    }
}

impl EncodePacket for LegacySetTitleAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.type_id().encode(writer, protocol_version)?;
        match self {
            LegacySetTitleAction::SetTitle { title } => {
                title.encode(writer, protocol_version)?;
            }
            LegacySetTitleAction::SetSubtitle { subtitle } => {
                subtitle.encode(writer, protocol_version)?;
            }
            LegacySetTitleAction::SetActionBar { action_bar } => {
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
            LegacySetTitleAction::Hide {} | LegacySetTitleAction::Reset {} => {
                // Nothing to encode
            }
        }
        Ok(())
    }
}

impl EncodePacket for LegacyV1_11SetTitleAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.type_id().encode(writer, protocol_version)?;
        match self {
            LegacyV1_11SetTitleAction::SetTitle { title } => {
                title.encode(writer, protocol_version)?;
            }
            LegacyV1_11SetTitleAction::SetSubtitle { subtitle } => {
                subtitle.encode(writer, protocol_version)?;
            }
            LegacyV1_11SetTitleAction::SetTimesAndDisplay {
                fade_in,
                stay,
                fade_out,
            } => {
                fade_in.encode(writer, protocol_version)?;
                stay.encode(writer, protocol_version)?;
                fade_out.encode(writer, protocol_version)?;
            }
            LegacyV1_11SetTitleAction::Hide {} | LegacyV1_11SetTitleAction::Reset {} => {
                // Nothing to encode
            }
        }
        Ok(())
    }
}
