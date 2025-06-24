use minecraft_protocol::prelude::*;
use thiserror::Error;

/// This packet is sent since 1.13
#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:commands")]
pub struct CommandsPacket {
    /// An array of nodes.
    nodes: LengthPaddedVec<Node>,
    /// Index of the `root` node in the previous array.
    root_index: VarInt,
}

impl CommandsPacket {
    pub fn empty() -> Self {
        Self {
            nodes: vec![Node {
                flags: 0,
                children: LengthPaddedVec::default(),
            }]
            .into(),
            root_index: VarInt::from(0),
        }
    }
}

#[derive(Debug, Default)]
struct Node {
    flags: i8,
    /// Array of indices of child nodes.
    children: LengthPaddedVec<VarInt>,
}

#[derive(Debug, Error)]
pub enum NodeError {
    #[error(transparent)]
    LengthPaddedVecEncode(#[from] LengthPaddedVecEncodeError),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
}

impl EncodePacketField for Node {
    type Error = NodeError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        self.flags.encode(bytes, protocol_version)?;
        self.children.encode(bytes, protocol_version)?;
        Ok(())
    }
}
