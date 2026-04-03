use crate::configuration::pick_item_config::PickItemConfig;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use blocks_report::load_internal_mapping;
use minecraft_packets::play::pick_item_from_block_packet::PickItemFromBlockPacket;
use minecraft_packets::play::set_player_inventory::SetPlayerInventoryPacket;
use minecraft_protocol::prelude::ProtocolVersion;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tracing::debug;

static INTERNAL_ID_TO_BLOCK_IDENTIFIER: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    let Ok(internal_mapping) = load_internal_mapping() else {
        return map;
    };

    for block in internal_mapping.mapping.inner() {
        for state in block.states.inner() {
            map.insert(state.state_data.internal_id(), block.name.clone());
        }
    }

    map
});

static ITEM_REGISTRY_CACHE: LazyLock<Mutex<HashMap<ProtocolVersion, HashMap<String, i32>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn data_generated_path(protocol_version: ProtocolVersion) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../data/generated")
        .join(protocol_version.packets().to_string())
        .join("reports/registries.json")
}

fn load_item_registry(protocol_version: ProtocolVersion) -> Option<HashMap<String, i32>> {
    let content = fs::read_to_string(data_generated_path(protocol_version)).ok()?;
    let value: JsonValue = serde_json::from_str(&content).ok()?;

    let entries = value.get("minecraft:item")?.get("entries")?.as_object()?;

    Some(
        entries
            .iter()
            .filter_map(|(item_identifier, item_data)| {
                let item_id = item_data.get("protocol_id")?.as_i64()?;
                let item_id = i32::try_from(item_id).ok()?;
                Some((item_identifier.clone(), item_id))
            })
            .collect(),
    )
}

fn get_item_id(protocol_version: ProtocolVersion, item_identifier: &str) -> Option<i32> {
    if let Ok(cache) = ITEM_REGISTRY_CACHE.lock()
        && let Some(item_registry) = cache.get(&protocol_version)
    {
        return item_registry.get(item_identifier).copied();
    }

    let item_registry = load_item_registry(protocol_version)?;
    let item_id = item_registry.get(item_identifier).copied();

    if let Ok(mut cache) = ITEM_REGISTRY_CACHE.lock() {
        cache.insert(protocol_version, item_registry);
    }

    item_id
}

fn push_unique(candidates: &mut Vec<String>, candidate: String) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

fn item_identifier_candidates(block_identifier: &str) -> Vec<String> {
    let mut candidates = vec![block_identifier.to_owned()];

    if let Some(rest) = block_identifier.strip_prefix("minecraft:potted_") {
        push_unique(&mut candidates, format!("minecraft:{rest}"));
    }

    if let Some(stem) = block_identifier.strip_suffix("_wall_torch") {
        push_unique(&mut candidates, format!("{stem}_torch"));
    }

    if let Some(stem) = block_identifier.strip_suffix("_wall_sign") {
        push_unique(&mut candidates, format!("{stem}_sign"));
    }

    if let Some(stem) = block_identifier.strip_suffix("_wall_hanging_sign") {
        push_unique(&mut candidates, format!("{stem}_hanging_sign"));
    }

    if block_identifier.ends_with("wall_head") {
        push_unique(
            &mut candidates,
            block_identifier.replacen("wall_head", "head", 1),
        );
    }

    if block_identifier.ends_with("wall_skull") {
        push_unique(
            &mut candidates,
            block_identifier.replacen("wall_skull", "skull", 1),
        );
    }

    candidates
}

fn resolve_item_id_for_block(
    protocol_version: ProtocolVersion,
    block_identifier: &str,
) -> Option<i32> {
    item_identifier_candidates(block_identifier)
        .into_iter()
        .find_map(|item_identifier| get_item_id(protocol_version, &item_identifier))
}

impl PacketHandler for PickItemFromBlockPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let config: &PickItemConfig = server_state.pick_item();

        if !config.enabled {
            return Ok(Batch::new());
        }

        let stack_size = config.stack_size;

        let mut batch = Batch::new();
        let current_inventory_slot = client_state.inventory().current_slot();

        let mut packet = SetPlayerInventoryPacket::empty();

        if let Some(world_context) = &server_state.world() {
            if let Some(internal_id) = world_context.get_block_at(
                self.location().x(),
                self.location().y(),
                self.location().z(),
            ) {
                let inventory_slot: i32 = current_inventory_slot as i32;

                packet.set_slot(inventory_slot);
                if let Some(block_identifier) = INTERNAL_ID_TO_BLOCK_IDENTIFIER.get(&internal_id) {
                    if let Some(item_id) =
                        resolve_item_id_for_block(client_state.protocol_version(), block_identifier)
                    {
                        packet.set_slot_data_from_id(item_id, stack_size);
                    } else {
                        debug!(
                            protocol_version = %client_state.protocol_version(),
                            block_identifier,
                            internal_id,
                            "Unable to resolve item id for pick block"
                        );
                    }
                }

                let _ = self.include_data();
            }
        }

        batch.queue(|| PacketRegistry::SetPlayerInventory(packet));

        Ok(batch)
    }
}
