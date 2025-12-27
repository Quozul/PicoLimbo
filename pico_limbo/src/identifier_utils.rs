pub fn to_protocol_identifier(
    iden: &pico_registries::Identifier,
) -> minecraft_protocol::prelude::Identifier {
    if iden.is_tag() {
        panic!("Tags identifier are not supported");
    }
    minecraft_protocol::prelude::Identifier::new(&iden.namespace, &iden.thing)
}

pub fn to_registries_identifier(
    iden: &minecraft_protocol::prelude::Identifier,
) -> pico_registries::Identifier {
    pico_registries::Identifier::new_unchecked(&iden.namespace, &iden.thing)
}
