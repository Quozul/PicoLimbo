use crate::data::packets_report::raw_packet_data::RawPacketData;
use crate::protocol_version::ProtocolVersion;
use crate::state::State;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock};

/// Custom error type for PacketMap operations.
#[derive(thiserror::Error, Debug)]
pub enum PacketMapError {
    #[error("Failed to read packet mapping file '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse packet mapping JSON: {source}")]
    Json {
        #[source]
        source: serde_json::Error,
    },
    #[error("Failed to acquire read lock on cached mappings: {0}")]
    ReadLockPoisoned(String),
    #[error("Failed to acquire write lock on cached mappings: {0}")]
    WriteLockPoisoned(String),
    #[error("Protocol version '{0}' reports path is empty or invalid")]
    InvalidProtocolReportsPath(u32),
}

impl<T> From<PoisonError<std::sync::RwLockReadGuard<'_, T>>> for PacketMapError {
    fn from(err: PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        PacketMapError::ReadLockPoisoned(err.to_string())
    }
}

impl<T> From<PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for PacketMapError {
    fn from(err: PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        PacketMapError::WriteLockPoisoned(err.to_string())
    }
}

type CachedMappings = Arc<RwLock<HashMap<i32, Arc<HashMap<String, u8>>>>>;

/// A mapping from composite packet names (like "handshake/serverbound/minecraft:intention")
/// to their packet IDs. This structure also caches the mapping per protocol version,
/// loading the JSON file from disk as needed.
#[derive(Clone)]
pub struct PacketMap {
    root_directory: PathBuf,
    // The cache holds the mapping for each protocol version.
    cached_mappings: CachedMappings,
}

impl PacketMap {
    /// Create a new `PacketMap` with the given root directory.
    ///
    /// The expected file structure is:
    /// `<root_directory>/<protocol_version>/reports/packets.json`
    pub fn new(root_directory: PathBuf) -> Self {
        Self {
            root_directory,
            cached_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the packet ID for the given protocol version and packet path.
    ///
    /// The packet path should be the full composite key, for example:
    /// `"handshake/serverbound/minecraft:intention"`.
    pub fn get_packet_id(
        &self,
        protocol_version: ProtocolVersion,
        packet_path: &'static str,
    ) -> Result<Option<u8>, PacketMapError> {
        let mapping = self.get_mapping(protocol_version)?;
        Ok(mapping.get(packet_path).copied())
    }

    /// Returns the packet name (the composite key) for the given protocol version,
    /// state, and packet ID.
    ///
    /// For example, if called with state `"handshake"` and packet ID `0`, it will
    /// look for a key starting with `"handshake/serverbound"` whose value is `0`.
    pub fn get_packet_name(
        &self,
        protocol_version: ProtocolVersion,
        state: &State,
        packet_id: u8,
    ) -> Result<Option<String>, PacketMapError> {
        let mapping = self.get_mapping(protocol_version)?;
        let prefix = format!("{state}/serverbound");
        Ok(mapping
            .iter()
            .find(|&(key, &id)| id == packet_id && key.starts_with(&prefix))
            .map(|(key, _)| key.clone()))
    }

    /// Retrieves (or loads and caches) the mapping for the specified protocol version.
    fn get_mapping(
        &self,
        protocol_version: ProtocolVersion,
    ) -> Result<Arc<HashMap<String, u8>>, PacketMapError> {
        {
            // Try a quick lookup under a read lock.
            let cache = self.cached_mappings.read()?;
            if let Some(mapping) = cache.get(&protocol_version.version_number()) {
                return Ok(mapping.clone());
            }
        }

        // Not cached yet: compute the file path and load the mapping.
        let file_path = self
            .root_directory
            .join(protocol_version.reports())
            .join("reports/packets.json");
        let mapping = Self::load_mapping_from_file(&file_path)?;
        let mapping_arc = Arc::new(mapping);

        // Cache the new mapping.
        let mut cache = self.cached_mappings.write()?;
        cache.insert(protocol_version.version_number(), mapping_arc.clone());
        Ok(mapping_arc)
    }

    /// Loads the mapping from the JSON file at the given path.
    fn load_mapping_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<String, u8>, PacketMapError> {
        let path_buf = path.as_ref().to_path_buf();
        let data = std::fs::read_to_string(&path_buf).map_err(|e| PacketMapError::Io {
            path: path_buf,
            source: e,
        })?;
        Self::parse_json(&data)
    }

    /// Parses the given JSON string into a mapping.
    ///
    /// The JSON is expected to have the following structure:
    ///
    /// ```json
    /// {
    ///   "state": {
    ///     "direction": {
    ///       "packet_name": { "protocol_id": number }
    ///     }
    ///   },
    ///   ...
    /// }
    /// ```
    ///
    /// For each packet, the composite key is built as:
    /// `"state/direction/packet_name"`.
    fn parse_json(json_str: &str) -> Result<HashMap<String, u8>, PacketMapError> {
        let raw_data: RawPacketData =
            serde_json::from_str(json_str).map_err(|e| PacketMapError::Json { source: e })?;

        let mut mapping = HashMap::new();

        for (state, directions) in raw_data.0 {
            for (direction, packets) in directions {
                for (packet_name, packet_info) in packets {
                    let key = format!("{state}/{direction}/{packet_name}");
                    mapping.insert(key, packet_info.protocol_id);
                }
            }
        }
        Ok(mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol_version::ProtocolVersion;
    use crate::state::State;
    use std::path::PathBuf;

    #[test]
    fn test_parse_json() {
        let json_str = r#"
        {
          "handshake": {
            "serverbound": {
              "minecraft:intention": {
                "protocol_id": 0
              }
            }
          },
          "login": {
            "clientbound": {
              "minecraft:game_profile": {
                "protocol_id": 2
              }
            },
            "serverbound": {
              "minecraft:hello": {
                "protocol_id": 0
              }
            }
          }
        }
        "#;

        let mapping = PacketMap::parse_json(json_str).unwrap();

        let key1 = "handshake/serverbound/minecraft:intention";
        assert_eq!(mapping.get(key1), Some(&0));

        let key2 = "login/serverbound/minecraft:hello";
        assert_eq!(mapping.get(key2), Some(&0));

        let key3 = "login/clientbound/minecraft:game_profile";
        assert_eq!(mapping.get(key3), Some(&2));
    }

    #[test]
    fn get_packet_id() {
        let packet_map = build_packet_map();
        let protocol_version = ProtocolVersion::V1_21_4;
        let packet_name = "handshake/serverbound/minecraft:intention";

        let packet_id = packet_map.get_packet_id(protocol_version, packet_name);
        assert_eq!(packet_id.unwrap().unwrap(), 0);
    }

    #[test]
    fn get_unknown_packet_id() {
        let packet_map = build_packet_map();
        let protocol_version = ProtocolVersion::V1_21_4;
        let packet_name = "handshake/serverbound/minecraft:foo";

        let packet_id = packet_map.get_packet_id(protocol_version, packet_name);
        assert!(packet_id.unwrap().is_none());
    }

    #[test]
    fn get_packet_name() {
        let packet_map = build_packet_map();
        let protocol_version = ProtocolVersion::V1_21_4;
        let state = State::Handshake;
        let packet_id = 0;

        let packet_name = packet_map.get_packet_name(protocol_version, &state, packet_id);
        assert_eq!(
            packet_name.unwrap().unwrap(),
            "handshake/serverbound/minecraft:intention".to_string()
        );
    }

    fn build_packet_map() -> PacketMap {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("../data/generated");
        PacketMap::new(data_dir)
    }
}
