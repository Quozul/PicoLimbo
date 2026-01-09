use minecraft_protocol::prelude::*;

#[derive(Eq, PartialEq, Clone, PacketOut, PacketIn, Hash, Ord, PartialOrd)]
pub struct InternalProperties {
    pub name: String,
    pub value: String,
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash, Copy, Clone)]
pub struct StateData {
    internal_id: InternalId,
    light: u8,
}

impl StateData {
    pub const fn new(internal_id: InternalId, is_transparent: bool, light_level: u8) -> Self {
        let light_level = light_level & 0x0F; // Mask to ensure 0-15
        let transparent_bit = if is_transparent { 0x10 } else { 0x00 };

        Self {
            internal_id,
            light: light_level | transparent_bit,
        }
    }

    /// Get the transparency flag
    pub const fn is_transparent(&self) -> bool {
        (self.light & 0x10) != 0
    }

    /// Get the light level (0-15)
    pub const fn get_emitted_light_level(&self) -> u8 {
        self.light & 0x0F
    }

    /// Get the internal id
    pub const fn internal_id(&self) -> InternalId {
        self.internal_id
    }
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash)]
pub struct InternalState {
    pub state_data: StateData,
    pub properties: LengthPaddedVec<InternalProperties>,
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash)]
pub struct InternalBlockMapping {
    pub name: String,
    pub states: LengthPaddedVec<InternalState>,
    pub default_state_data: StateData,
}

#[derive(PacketOut, PacketIn)]
pub struct InternalMapping {
    pub mapping: LengthPaddedVec<InternalBlockMapping>,
}

pub type InternalId = u16;
