use serde::Deserialize;
use std::collections::HashMap;

/// Represents the "protocol_id" object within the JSON.
#[derive(Deserialize)]
pub struct PacketInfo {
    pub protocol_id: u8,
}

/// Represents the mapping from packet_name to PacketInfo.
pub type DirectionPackets = HashMap<String, PacketInfo>;

/// Represents the mapping from direction (serverbound/clientbound) to DirectionPackets.
pub type StateDirections = HashMap<String, DirectionPackets>;

/// Represents the top-level structure of the JSON: state -> directions -> packets.
#[derive(Deserialize)]
pub struct RawPacketData(pub HashMap<String, StateDirections>);
