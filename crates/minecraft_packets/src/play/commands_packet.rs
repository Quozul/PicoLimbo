use minecraft_protocol::prelude::*;

/// This packet is sent since 1.13
#[derive(PacketOut)]
pub struct CommandsPacket {
    /// An array of nodes.
    nodes: LengthPaddedVec<Node>,
    /// Index of the `root` node in the previous array.
    root_index: VarInt,
}

const ROOT_NODE: i8 = NodeFlagsBuilder::new().node_type(NodeType::Root).build();

const LITERAL_NODE: i8 = NodeFlagsBuilder::new()
    .node_type(NodeType::Literal)
    .executable(true)
    .build();

impl CommandsPacket {
    pub fn spawn_command() -> Self {
        let child_nodes = vec![Node::literal("spawn")];

        let root_children_indices: Vec<i32> = (1..=child_nodes.len()).map(|i| i as i32).collect();

        let root_node = Node::root(root_children_indices);

        let mut nodes = Vec::with_capacity(1 + child_nodes.len());
        nodes.push(root_node);
        nodes.extend(child_nodes);

        Self {
            nodes: LengthPaddedVec::new(nodes),
            root_index: VarInt::from(0),
        }
    }

    pub fn empty() -> Self {
        Self {
            nodes: LengthPaddedVec::new(vec![Node::root(vec![])]),
            root_index: VarInt::from(0),
        }
    }
}

#[derive(PacketOut)]
struct Node {
    flags: i8,
    /// Array of indices of child nodes.
    children: LengthPaddedVec<VarInt>,
    /// Only for literal nodes
    name: Omitted<String>,
}

impl Node {
    fn root(children: Vec<i32>) -> Self {
        Node {
            flags: ROOT_NODE,
            children: LengthPaddedVec::new(children.iter().map(VarInt::from).collect()),
            name: Omitted::None,
        }
    }

    fn literal(name: impl ToString) -> Self {
        Node {
            flags: LITERAL_NODE,
            children: LengthPaddedVec::default(),
            name: Omitted::Some(name.to_string()),
        }
    }
}

enum NodeType {
    Root = 0,
    Literal = 1,
}

pub struct NodeFlagsBuilder {
    flags: i8,
}

impl NodeFlagsBuilder {
    const fn new() -> Self {
        Self { flags: 0 }
    }

    /// 0: root, 1: literal, 2: argument. 3 is not used.
    const fn node_type(mut self, node_type: NodeType) -> Self {
        self.flags = (self.flags & !0x03) | (node_type as i8);
        self
    }

    /// Set if the node stack to this point constitutes a valid command.
    const fn executable(mut self, value: bool) -> Self {
        if value {
            self.flags |= 0x04;
        } else {
            self.flags &= !0x04;
        }
        self
    }

    const fn build(self) -> i8 {
        self.flags
    }
}
