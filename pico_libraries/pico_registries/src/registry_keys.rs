use pico_identifier::prelude::Identifier;
use protocol_version::protocol_version::ProtocolVersion;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Only absolute mandatory registry keys are mapped for now
#[derive(Hash, Eq, PartialEq, Clone)]
pub enum RegistryKeys {
    Root,
    Biome,
    CatVariant,
    ChickenVariant,
    CowVariant,
    DamageType,
    Dialog,
    DimensionType,
    FrogVariant,
    PaintingVariant,
    PigVariant,
    Timeline,
    WolfSoundVariant,
    WolfVariant,
    ZombieNautilusVariant,
    Custom(Identifier),
}

impl RegistryKeys {
    #[must_use]
    pub fn id(&self) -> Identifier {
        match self {
            Self::Root => Identifier::vanilla_unchecked("root"),
            Self::Biome => Identifier::vanilla_unchecked("worldgen/biome"),
            Self::CatVariant => Identifier::vanilla_unchecked("cat_variant"),
            Self::ChickenVariant => Identifier::vanilla_unchecked("chicken_variant"),
            Self::CowVariant => Identifier::vanilla_unchecked("cow_variant"),
            Self::DamageType => Identifier::vanilla_unchecked("damage_type"),
            Self::Dialog => Identifier::vanilla_unchecked("dialog"),
            Self::DimensionType => Identifier::vanilla_unchecked("dimension_type"),
            Self::FrogVariant => Identifier::vanilla_unchecked("frog_variant"),
            Self::PaintingVariant => Identifier::vanilla_unchecked("painting_variant"),
            Self::PigVariant => Identifier::vanilla_unchecked("pig_variant"),
            Self::Timeline => Identifier::vanilla_unchecked("timeline"),
            Self::WolfSoundVariant => Identifier::vanilla_unchecked("wolf_sound_variant"),
            Self::WolfVariant => Identifier::vanilla_unchecked("wolf_variant"),
            Self::ZombieNautilusVariant => Identifier::vanilla_unchecked("zombie_nautilus_variant"),
            Self::Custom(identifier) => identifier.clone(),
        }
    }

    #[must_use]
    pub const fn is_mandatory(&self) -> bool {
        matches!(
            self,
            Self::Biome
                | Self::CatVariant
                | Self::ChickenVariant
                | Self::CowVariant
                | Self::DamageType
                | Self::Dialog
                | Self::DimensionType
                | Self::FrogVariant
                | Self::PaintingVariant
                | Self::PigVariant
                | Self::Timeline
                | Self::WolfSoundVariant
                | Self::WolfVariant
                | Self::ZombieNautilusVariant
        )
    }

    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    #[must_use]
    pub fn get_tag_path(&self) -> String {
        format!("tags/{}", self.id().thing)
    }

    #[must_use]
    pub const fn get_minimum_version(&self) -> Option<ProtocolVersion> {
        match self {
            Self::Biome => Some(ProtocolVersion::V1_16_2),
            Self::CatVariant
            | Self::ChickenVariant
            | Self::CowVariant
            | Self::FrogVariant
            | Self::PigVariant
            | Self::WolfSoundVariant => Some(ProtocolVersion::V1_21_5),
            Self::DamageType => Some(ProtocolVersion::V1_19_4),
            Self::Dialog => Some(ProtocolVersion::V1_21_6),
            Self::DimensionType => Some(ProtocolVersion::V1_16),
            Self::PaintingVariant => Some(ProtocolVersion::V1_21),
            Self::Timeline | Self::ZombieNautilusVariant => Some(ProtocolVersion::V1_21_11),
            Self::WolfVariant => Some(ProtocolVersion::V1_20_5),

            _ => None,
        }
    }
}

impl Display for RegistryKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.id().to_string().as_str())
    }
}

impl fmt::Debug for RegistryKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.id().to_string().as_str())
    }
}
