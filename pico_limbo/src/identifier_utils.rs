use minecraft_protocol::prelude::Dimension as ProtocolDimension;
use pico_registries::utils::dimension_codec::Dimension as RegistryDimension;

pub fn to_protocol_identifier(
    iden: &pico_registries::Identifier,
) -> minecraft_protocol::prelude::Identifier {
    assert!(!iden.is_tag(), "Tags identifier are not supported");
    minecraft_protocol::prelude::Identifier::new(&iden.namespace, &iden.thing)
}

pub fn to_registries_identifier(
    iden: &minecraft_protocol::prelude::Identifier,
) -> pico_registries::Identifier {
    pico_registries::Identifier::new_unchecked(&iden.namespace, &iden.thing)
}

pub const fn to_registries_dimension(dim: ProtocolDimension) -> RegistryDimension {
    match dim {
        ProtocolDimension::Overworld => RegistryDimension::Overworld,
        ProtocolDimension::Nether => RegistryDimension::TheNether,
        ProtocolDimension::End => RegistryDimension::TheEnd,
    }
}
