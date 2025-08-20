use minecraft_protocol::prelude::*;

/// This packet is sent since 1.13
#[derive(PacketOut)]
pub struct CommandsPacket {
    /// An array of nodes.
    nodes: LengthPaddedVec<Node>,
    /// Index of the `root` node in the previous array.
    root_index: VarInt,
}

impl CommandsPacket {
    pub fn empty() -> Self {
        Self {
            nodes: LengthPaddedVec::new(vec![Node {
                flags: 0,
                children: LengthPaddedVec::default(),
            }]),
            root_index: VarInt::from(0),
        }
    }
}

#[derive(PacketOut)]
struct Node {
    flags: i8,
    /// Array of indices of child nodes.
    children: LengthPaddedVec<VarInt>,
}
