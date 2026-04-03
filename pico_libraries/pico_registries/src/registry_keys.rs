use macros::RegistryKeys;
use pico_identifier::prelude::Identifier;
use protocol_version::protocol_version::ProtocolVersion;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Only absolute mandatory registry keys are mapped for now
#[derive(Hash, Eq, PartialEq, Clone, RegistryKeys)]
pub enum RegistryKeys {
    #[registry(root)]
    Root,
    #[registry(id = "minecraft:banner_pattern", min_version = V26_1, is_mandatory = true)]
    BannerPattern,
    #[registry(id = "minecraft:cat_sound_variant", min_version = V26_1, is_mandatory = true)]
    CatSoundVariant,
    #[registry(id = "minecraft:cat_variant", min_version = V1_21_5, is_mandatory = true)]
    CatVariant,
    #[registry(id = "minecraft:chicken_sound_variant", min_version = V26_1, is_mandatory = true)]
    ChickenSoundVariant,
    #[registry(id = "minecraft:chicken_variant", min_version = V1_21_5, is_mandatory = true)]
    ChickenVariant,
    #[registry(id = "minecraft:cow_sound_variant", min_version = V26_1, is_mandatory = true)]
    CowSoundVariant,
    #[registry(id = "minecraft:cow_variant", min_version = V1_21_5, is_mandatory = true)]
    CowVariant,
    #[registry(id = "minecraft:damage_type", min_version = V1_19_4, is_mandatory = true)]
    DamageType,
    #[registry(id = "minecraft:dialog", min_version = V1_21_6, is_mandatory = true)]
    Dialog,
    #[registry(id = "minecraft:dimension_type", min_version = V1_16, is_mandatory = true)]
    DimensionType,
    #[registry(id = "minecraft:frog_variant", min_version = V1_21_5, is_mandatory = true)]
    FrogVariant,
    #[registry(id = "minecraft:instrument", min_version = V26_1, is_mandatory = true)]
    Instrument,
    #[registry(id = "minecraft:jukebox_song", min_version = V26_1, is_mandatory = true)]
    JukeboxSong,
    #[registry(id = "minecraft:painting_variant", min_version = V1_21, is_mandatory = true)]
    PaintingVariant,
    #[registry(id = "minecraft:pig_sound_variant", min_version = V26_1, is_mandatory = true)]
    PigSoundVariant,
    #[registry(id = "minecraft:pig_variant", min_version = V1_21_5, is_mandatory = true)]
    PigVariant,
    #[registry(id = "minecraft:timeline", min_version = V1_21_11, is_mandatory = true)]
    Timeline,
    #[registry(id = "minecraft:trim_material", min_version = V26_1, is_mandatory = true)]
    TrimMaterial,
    #[registry(id = "minecraft:wolf_sound_variant", min_version = V1_21_5, is_mandatory = true)]
    WolfSoundVariant,
    #[registry(id = "minecraft:wolf_variant", min_version = V1_20_5, is_mandatory = true)]
    WolfVariant,
    #[registry(id = "minecraft:world_clock", min_version = V26_1, is_mandatory = true)]
    WorldClock,
    #[registry(id = "minecraft:worldgen/biome", min_version = V1_16_2, is_mandatory = true)]
    WorldGenBiome,
    #[registry(id = "minecraft:zombie_nautilus_variant", min_version = V1_21_11, is_mandatory = true)]
    ZombieNautilusVariant,
    #[registry(custom)]
    Custom(Identifier),
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
