use crate::data::registry_entry::RegistryEntry;
use crate::{Registry, RegistryKeys, RegistryManager};
use pico_identifier::Identifier;
use std::borrow::Cow;

pub struct TaggedRegistry {
    pub registry_id: Identifier,
    pub tags: Vec<RegistryTag>,
}

pub struct RegistryTag {
    pub identifier: Identifier,
    /// List of protocol IDs
    pub ids: Cow<'static, [u32]>,
}

pub fn get_tagged_registries(registry_manager: &RegistryManager) -> Vec<TaggedRegistry> {
    let tag_registries = &[RegistryKeys::Timeline];

    tag_registries
        .iter()
        .filter_map(|registry_keys| registry_manager.try_get(registry_keys))
        .flat_map(|registry| -> crate::Result<TaggedRegistry> {
            let tags = registry.get_tag_identifiers();
            let registry_identifier = registry.get_registry_key().get_value();

            Ok(TaggedRegistry {
                registry_id: registry_identifier.clone(),
                tags: tags
                    .iter()
                    .flat_map(|tag_name| -> crate::Result<RegistryTag> {
                        Ok(RegistryTag {
                            identifier: tag_name.normalize(),
                            ids: Cow::Owned(evaluate_tags(registry, tag_name)?),
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

// This function is called recursively
fn evaluate_tags(registry: &Registry, tag_name: &Identifier) -> crate::Result<Vec<u32>> {
    Ok(registry
        .get_tag(tag_name)?
        .get_values()
        .iter()
        .flat_map(|identifier| {
            if identifier.is_tag() {
                // If it is a tag, we should expend all the values from that tag into the current tag
                evaluate_tags(registry, &identifier.normalize())
            } else {
                // If it is not a tag, then we should get the protocol ID of the actual value from the registry
                Ok(registry
                    .try_get(identifier)
                    .into_iter()
                    .map(RegistryEntry::get_protocol_id)
                    .collect())
            }
        })
        .flatten()
        .collect())
}
