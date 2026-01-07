use minecraft_protocol::prelude::*;

const ROOT_NODE: i8 = NodeFlagsBuilder::new().node_type(NodeType::Root).build();

const LITERAL_NODE: i8 = NodeFlagsBuilder::new()
    .node_type(NodeType::Literal)
    .executable(true)
    .build();

const ARGUMENT_NODE: i8 = NodeFlagsBuilder::new()
    .node_type(NodeType::Argument)
    .executable(true)
    .build();

/// This packet is sent since 1.13
#[derive(PacketOut)]
pub struct CommandsPacket {
    /// An array of nodes.
    nodes: LengthPaddedVec<Node>,
    /// Index of the `root` node in the previous array.
    root_index: VarInt,
}

pub enum CommandArgumentType {
    Float { min: f32, max: f32 },
}

pub struct CommandArgument {
    name: String,
    argument_type: CommandArgumentType,
}

impl CommandArgument {
    pub fn float(name: impl ToString, min: f32, max: f32) -> Self {
        Self {
            name: name.to_string(),
            argument_type: CommandArgumentType::Float { min, max },
        }
    }
}

pub struct Command {
    alias: String,
    arguments: Vec<CommandArgument>,
}

impl Command {
    pub fn new(alias: impl ToString, arguments: Vec<CommandArgument>) -> Self {
        Self {
            alias: alias.to_string(),
            arguments,
        }
    }

    pub fn no_arguments(alias: impl ToString) -> Self {
        Self {
            alias: alias.to_string(),
            arguments: Vec::new(),
        }
    }
}

impl CommandsPacket {
    pub fn new(commands: Vec<Command>) -> Self {
        let mut nodes = vec![Node::root(vec![])];

        let mut root_children_indices = Vec::new();

        for command in commands {
            let mut current_node_index = nodes.len() as i32;
            root_children_indices.push(current_node_index);

            nodes.push(Node::literal(command.alias));

            for argument in command.arguments {
                let argument_node_index = nodes.len() as i32;

                if let Some(previous_node) = nodes.get_mut(current_node_index as usize) {
                    previous_node.add_child_index(argument_node_index);
                }

                let properties = match argument.argument_type {
                    CommandArgumentType::Float { min, max } => ParserProperties::float(min, max),
                };

                nodes.push(Node::argument(argument.name, properties));

                current_node_index = argument_node_index;
            }
        }

        if let Some(root) = nodes.get_mut(0) {
            for index in root_children_indices {
                root.add_child_index(index);
            }
        }

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
    data: NodeData,
}

impl Node {
    fn root(children: Vec<i32>) -> Self {
        Node {
            flags: ROOT_NODE,
            children: LengthPaddedVec::new(children.iter().map(VarInt::from).collect()),
            data: NodeData::Root,
        }
    }

    fn literal(name: impl ToString) -> Self {
        Node {
            flags: LITERAL_NODE,
            children: LengthPaddedVec::default(),
            data: NodeData::Literal {
                name: name.to_string(),
            },
        }
    }

    fn argument(name: impl ToString, parser_properties: ParserProperties) -> Self {
        Node {
            flags: ARGUMENT_NODE,
            children: LengthPaddedVec::default(),
            data: NodeData::Argument {
                name: name.to_string(),
                properties: parser_properties,
            },
        }
    }

    fn add_child_index(&mut self, child: i32) -> &mut Self {
        self.children.inner_mut().push(VarInt::new(child));
        self
    }
}

enum NodeData {
    Root,
    Literal {
        name: String,
    },
    Argument {
        name: String,
        properties: ParserProperties,
    },
}

impl EncodePacket for NodeData {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            NodeData::Root => {}
            NodeData::Literal { name } => {
                name.encode(writer, protocol_version)?;
            }
            NodeData::Argument { name, properties } => {
                name.encode(writer, protocol_version)?;
                properties.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}

enum ParserProperties {
    Float {
        flags: i8,
        /// Only if flags & 0x01. If not specified, defaults to -Float.MAX_VALUE (≈ 3.4028235E38)
        min: Omitted<f32>,
        /// Only if flags & 0x02. If not specified, defaults to Float.MAX_VALUE (≈ 3.4028235E38)
        max: Omitted<f32>,
    },
}

impl ParserProperties {
    fn id(&self) -> VarInt {
        match self {
            Self::Float { .. } => VarInt::new(1),
        }
    }

    fn identifier(&self) -> Identifier {
        match self {
            ParserProperties::Float { .. } => Identifier::new("brigadier", "float"),
        }
    }

    fn float(min: f32, max: f32) -> Self {
        Self::Float {
            flags: 0x01 | 0x02,
            min: Omitted::Some(min),
            max: Omitted::Some(max),
        }
    }
}

impl EncodePacket for ParserProperties {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
            self.id().encode(writer, protocol_version)?
        } else {
            self.identifier().encode(writer, protocol_version)?
        }

        match self {
            ParserProperties::Float { flags, min, max } => {
                flags.encode(writer, protocol_version)?;
                min.encode(writer, protocol_version)?;
                max.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}

#[repr(i8)]
enum NodeType {
    Root = 0,
    Literal = 1,
    Argument = 2,
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
