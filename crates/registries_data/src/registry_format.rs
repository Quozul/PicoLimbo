use minecraft_protocol::prelude::ProtocolVersion;

pub enum RegistryFormat {
    /// For all versions after 1.20.5 included
    V1_20_5,
    /// For all versions between 1.20.2 and 1.20.3 included
    /// This format is actually the same as 1.19, however, the registries are now sent
    /// in the configuration state instead of the play state
    V1_20_2,
    /// For all versions between 1.19 and 1.20 included
    V1_19,
    /// For all versions between 1.16.2 and 1.18.2 included
    V1_16_2,
    /// For all versions between 1.16 and 1.16.1 included
    V1_16,
    /// For versions prior to 1.16 excluded
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
