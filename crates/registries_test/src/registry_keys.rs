use minecraft_protocol::prelude::Identifier;

/// Only absolute mandatory registry keys are mapped for now
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum RegistryKeys {
    Root,
    Biome,
    CatVariant,
    ChickenVariant,
    CowVariant,
    DamageType,
    DimensionType,
    FrogVariant,
    PaintingVariant,
    PigVariant,
    WolfSoundVariant,
    WolfVariant,
}

impl RegistryKeys {
    pub fn id(&self) -> Identifier {
        match self {
            RegistryKeys::Root => Identifier::minecraft("root"),
            RegistryKeys::Biome => Identifier::minecraft("worldgen/biome"),
            RegistryKeys::CatVariant => Identifier::minecraft("cat_variant"),
            RegistryKeys::ChickenVariant => Identifier::minecraft("chicken_variant"),
            RegistryKeys::CowVariant => Identifier::minecraft("cow_variant"),
            RegistryKeys::DamageType => Identifier::minecraft("damage_type"),
            RegistryKeys::DimensionType => Identifier::minecraft("dimension_type"),
            RegistryKeys::FrogVariant => Identifier::minecraft("frog_variant"),
            RegistryKeys::PaintingVariant => Identifier::minecraft("painting_variant"),
            RegistryKeys::PigVariant => Identifier::minecraft("pig_variant"),
            RegistryKeys::WolfSoundVariant => Identifier::minecraft("wolf_sound_variant"),
            RegistryKeys::WolfVariant => Identifier::minecraft("wolf_variant"),
        }
    }

    pub fn is_mandatory(&self) -> bool {
        matches!(
            self,
            RegistryKeys::Biome
                | RegistryKeys::CatVariant
                | RegistryKeys::ChickenVariant
                | RegistryKeys::CowVariant
                | RegistryKeys::DamageType
                | RegistryKeys::DimensionType
                | RegistryKeys::FrogVariant
                | RegistryKeys::PaintingVariant
                | RegistryKeys::PigVariant
                | RegistryKeys::WolfSoundVariant
                | RegistryKeys::WolfVariant
        )
    }

    pub fn is_root(&self) -> bool {
        matches!(self, RegistryKeys::Root)
    }
}
