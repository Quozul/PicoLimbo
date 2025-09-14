use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct SetTitlePacket {
    action: SetTitleAction,
}

impl SetTitlePacket {
    pub fn set_title(title: &Component) -> Self {
        Self {
            action: SetTitleAction::SetTitle {
                title: title.clone(),
            },
        }
    }

    pub fn set_subtitle(subtitle: &Component) -> Self {
        Self {
            action: SetTitleAction::SetSubtitle {
                subtitle: subtitle.clone(),
            },
        }
    }

    pub fn set_action_bar(action_bar: &Component) -> Self {
        Self {
            action: SetTitleAction::SetActionBar {
                action_bar: action_bar.clone(),
            },
        }
    }

    pub fn set_times_and_display(fade_in: i32, stay: i32, fade_out: i32) -> Self {
        Self {
            action: SetTitleAction::SetTimesAndDisplay {
                fade_in,
                stay,
                fade_out,
            },
        }
    }

    pub fn hide() -> Self {
        Self {
            action: SetTitleAction::Hide {},
        }
    }

    pub fn reset() -> Self {
        Self {
            action: SetTitleAction::Reset {},
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
            Self::set_times_and_display(fade_in, stay, fade_out),
            Self::set_title(title),
            Self::set_subtitle(subtitle),
        ]
    }
}

#[allow(dead_code)]
enum SetTitleAction {
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

impl SetTitleAction {
    fn type_id(&self) -> u8 {
        match self {
            SetTitleAction::SetTitle { .. } => 0,
            SetTitleAction::SetSubtitle { .. } => 1,
            SetTitleAction::SetActionBar { .. } => 2,
            SetTitleAction::SetTimesAndDisplay { .. } => 3,
            SetTitleAction::Hide {} => 4,
            SetTitleAction::Reset {} => 5,
        }
    }
}

impl EncodePacket for SetTitleAction {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.type_id().encode(writer, protocol_version)?;
        match self {
            SetTitleAction::SetTitle { title } => {
                title.encode(writer, protocol_version)?;
            }
            SetTitleAction::SetSubtitle { subtitle } => {
                subtitle.encode(writer, protocol_version)?;
            }
            SetTitleAction::SetActionBar { action_bar } => {
                action_bar.encode(writer, protocol_version)?;
            }
            SetTitleAction::SetTimesAndDisplay {
                fade_in,
                stay,
                fade_out,
            } => {
                fade_in.encode(writer, protocol_version)?;
                stay.encode(writer, protocol_version)?;
                fade_out.encode(writer, protocol_version)?;
            }
            SetTitleAction::Hide {} => {
                // Nothing to encode
            }
            SetTitleAction::Reset {} => {
                // Nothing to encode
            }
        }
        Ok(())
    }
}
