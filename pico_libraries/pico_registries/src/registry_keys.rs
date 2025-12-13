use pico_identifier::prelude::Identifier;

/// Only absolute mandatory registry keys are mapped for now
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum RegistryKeys {
    Root,
    BannerPattern,
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
    Timeline,
    ZombieNautilusVariant,
    Biome,
    Custom(Identifier),
}

impl RegistryKeys {
    pub fn id(&self) -> Identifier {
        match self {
            Self::Root => Identifier::vanilla_unchecked("root"),
            Self::BannerPattern => Identifier::vanilla_unchecked("banner_pattern"),
            Self::CatVariant => Identifier::vanilla_unchecked("cat_variant"),
            Self::ChickenVariant => Identifier::vanilla_unchecked("chicken_variant"),
            Self::CowVariant => Identifier::vanilla_unchecked("cow_variant"),
            Self::DamageType => Identifier::vanilla_unchecked("damage_type"),
            Self::DimensionType => Identifier::vanilla_unchecked("dimension_type"),
            Self::FrogVariant => Identifier::vanilla_unchecked("frog_variant"),
            Self::PaintingVariant => Identifier::vanilla_unchecked("painting_variant"),
            Self::PigVariant => Identifier::vanilla_unchecked("pig_variant"),
            Self::WolfSoundVariant => Identifier::vanilla_unchecked("wolf_sound_variant"),
            Self::WolfVariant => Identifier::vanilla_unchecked("wolf_variant"),
            Self::Timeline => Identifier::vanilla_unchecked("timeline"),
            Self::ZombieNautilusVariant => Identifier::vanilla_unchecked("zombie_nautilus_variant"),
            Self::Biome => Identifier::vanilla_unchecked("worldgen/biome"),
            Self::Custom(identifier) => identifier.clone(),
        }
    }

    pub const fn is_mandatory(&self) -> bool {
        matches!(
            self,
            Self::Biome
                | Self::CatVariant
                | Self::ChickenVariant
                | Self::CowVariant
                | Self::DamageType
                | Self::DimensionType
                | Self::FrogVariant
                | Self::PaintingVariant
                | Self::PigVariant
                | Self::WolfSoundVariant
                | Self::WolfVariant
                | Self::ZombieNautilusVariant
        )
    }

    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    pub fn get_tag_path(&self) -> String {
        format!("tags/{}", self.id().thing)
    }
}
