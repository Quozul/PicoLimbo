use minecraft_protocol::prelude::ProtocolVersion;

pub enum RegistryFormat {
    V1_20_5,
    V1_20_2,
    V1_19,
    V1_16_2,
    V1_16,
    None,
}

impl RegistryFormat {
    pub fn from_version(protocol_version: ProtocolVersion) -> Self {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
            Self::V1_20_5
        } else if protocol_version
            .between_inclusive(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3)
        {
            Self::V1_20_2
        } else if protocol_version.between_inclusive(ProtocolVersion::V1_19, ProtocolVersion::V1_20)
        {
            Self::V1_19
        } else if protocol_version
            .between_inclusive(ProtocolVersion::V1_16_2, ProtocolVersion::V1_18_2)
        {
            Self::V1_16_2
        } else if protocol_version
            .between_inclusive(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1)
        {
            Self::V1_16
        } else {
            Self::None
        }
    }
}
