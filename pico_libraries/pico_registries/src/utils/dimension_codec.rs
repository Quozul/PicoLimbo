use crate::RegistryKeys;
use crate::utils::shared::{
    encode_nameless_compound_to_bytes, get_dimension, load_registry_manager,
};
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;

pub enum Dimension {
    Overworld,
    TheNether,
    TheEnd,
}

impl Dimension {
    #[must_use]
    pub fn identifier(&self) -> Identifier {
        match self {
            Self::Overworld => Identifier::vanilla_unchecked("overworld"),
            Self::TheNether => Identifier::vanilla_unchecked("the_nether"),
            Self::TheEnd => Identifier::vanilla_unchecked("the_end"),
        }
    }
}

/// Dimension codec is a thing from 1.16.2 up to 1.18.2
///
/// # Returns
/// Serialized NBT of the dimension codec
///
/// # Errors
pub fn get_dimension_codec_v1_16_2(
    protocol_version: ProtocolVersion,
    dimension: &Dimension,
) -> crate::Result<Vec<u8>> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16_2,
        ProtocolVersion::V1_18_2,
    )?;
    let registry_manager = load_registry_manager(protocol_version, &[RegistryKeys::DimensionType])?;
    let entry = get_dimension(&registry_manager, &dimension.identifier())?;
    Ok(encode_nameless_compound_to_bytes(
        protocol_version,
        &entry.get_raw_value(),
    )?)
}
