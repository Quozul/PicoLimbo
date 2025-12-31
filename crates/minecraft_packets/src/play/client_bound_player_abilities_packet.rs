use minecraft_protocol::prelude::*;

/// The latter 2 floats are used to indicate the flying speed and field of view respectively, while the first byte is used to determine the value of 4 booleans.
/// If Flying is set but Allow Flying is unset, the player is unable to stop flying.
#[derive(PacketOut)]
pub struct ClientBoundPlayerAbilitiesPacket {
    /// Invulnerable: 0x01
    /// Flying: 0x02
    /// Allow Flying: 0x04
    /// Creative Mode (Instant Break): 0x08
    flags: i8,
    /// 0.05 by default.
    flying_speed: f32,
    /// Modifies the field of view, like a speed potion. A vanilla server will use the same value as the movement speed sent in the Update Attributes packet, which defaults to 0.1 for players.
    field_of_view_modifier: f32,
}

impl ClientBoundPlayerAbilitiesPacket {
    pub fn builder() -> PlayerAbilitiesPacketBuilder {
        PlayerAbilitiesPacketBuilder::default()
    }
}

impl Default for ClientBoundPlayerAbilitiesPacket {
    fn default() -> Self {
        Self {
            flags: 0,
            flying_speed: 0.05,
            field_of_view_modifier: 0.1,
        }
    }
}

#[derive(Default)]
pub struct PlayerAbilitiesPacketBuilder {
    invulnerable: bool,
    flying: bool,
    allow_flying: bool,
    creative: bool,
    flying_speed: f32,
    field_of_view_modifier: f32,
}

impl PlayerAbilitiesPacketBuilder {
    const INVULNERABLE: i8 = 0x01;
    const FLYING: i8 = 0x02;
    const ALLOW_FLYING: i8 = 0x04;
    const CREATIVE: i8 = 0x08;

    pub fn new() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            allow_flying: false,
            creative: false,
            flying_speed: 0.05,
            field_of_view_modifier: 0.1,
        }
    }

    pub fn invulnerable(mut self, invulnerable: bool) -> Self {
        self.invulnerable = invulnerable;
        self
    }

    pub fn flying(mut self, flying: bool) -> Self {
        self.flying = flying;
        self
    }

    pub fn allow_flying(mut self, allow_flying: bool) -> Self {
        self.allow_flying = allow_flying;
        self
    }

    pub fn creative(mut self, creative: bool) -> Self {
        self.creative = creative;
        self
    }

    pub fn flying_speed(mut self, flying_speed: f32) -> Self {
        self.flying_speed = flying_speed;
        self
    }

    pub fn field_of_view_modifier(mut self, field_of_view_modifier: f32) -> Self {
        self.field_of_view_modifier = field_of_view_modifier;
        self
    }

    pub fn build(self) -> ClientBoundPlayerAbilitiesPacket {
        let mut flags = 0i8;

        if self.invulnerable {
            flags |= Self::INVULNERABLE;
        }
        if self.flying {
            flags |= Self::FLYING;
        }
        if self.allow_flying {
            flags |= Self::ALLOW_FLYING;
        }
        if self.creative {
            flags |= Self::CREATIVE;
        }

        ClientBoundPlayerAbilitiesPacket {
            flags,
            flying_speed: self.flying_speed,
            field_of_view_modifier: self.field_of_view_modifier,
        }
    }
}
