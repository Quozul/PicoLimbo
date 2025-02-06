use data_types::prelude::VarInt;
use std::cmp::Ordering;

#[derive(Debug, Clone, Default)]
pub enum ProtocolVersion {
    #[default]
    V1_21_4,
    /// Same protocol version number for 1.21.3
    V1_21_2,
    /// Same protocol version number for 1.21.1
    V1_21,
    /// Same protocol version number for 1.20.6
    V1_20_5,
    /// Same protocol version number for 1.20.4
    V1_20_3,
    V1_20_2,
    /// Same protocol version number for 1.20.1
    V1_20,
    V1_19_4,
    V1_19_3,
    /// Same protocol version number for 1.19.2
    V1_19_1,
    V1_19,
    V1_18_2,
    /// Same protocol version number for 1.18.1
    V1_18,
    V1_17_1,
    V1_17,
    /// Same protocol version number for 1.16.5
    V1_16_4,
    V1_16_3,
    V1_16_2,
    V1_16_1,
    V1_16,
    V1_15_2,
    V1_15_1,
    V1_15,
    V1_14_4,
    V1_14_3,
    V1_14_2,
    V1_14_1,
    V1_14,
    V1_13_2,
    V1_13_1,
    V1_13,
    V1_12_2,
    V1_12_1,
    V1_12,
    /// Same protocol version number for 1.11.2
    V1_11_1,
    V1_11,
    /// Same protocol version number for 1.10.1 and 1.10.2
    V1_10,
    /// Same protocol version number for 1.9.4
    V1_9_3,
    V1_9_2,
    V1_9_1,
    V1_9,
    /// Same protocol version number for 1.8.1, 1.8.2, 1.8.3, 1.8.4, 1.8.5, 1.8.6, 1.8.7, 1.8.8 and 1.8.9
    V1_8,
    /// Same protocol version number for 1.7.7, 1.7.8, 1.7.9 and 1.7.10
    V1_7_6,
    /// Same protocol version number for 1.7.2, 1.7.4 and 1.7.5
    V1_7_2,
}

impl ProtocolVersion {
    pub const fn version_number(&self) -> u32 {
        match self {
            ProtocolVersion::V1_21_4 => 769,
            ProtocolVersion::V1_21_2 => 768,
            ProtocolVersion::V1_21 => 767,
            ProtocolVersion::V1_20_5 => 766,
            ProtocolVersion::V1_20_3 => 765,
            ProtocolVersion::V1_20_2 => 764,
            ProtocolVersion::V1_20 => 763,
            ProtocolVersion::V1_19_4 => 762,
            ProtocolVersion::V1_19_3 => 761,
            ProtocolVersion::V1_19_1 => 760,
            ProtocolVersion::V1_19 => 759,
            ProtocolVersion::V1_18_2 => 758,
            ProtocolVersion::V1_18 => 757,
            ProtocolVersion::V1_17_1 => 756,
            ProtocolVersion::V1_17 => 755,
            ProtocolVersion::V1_16_4 => 754,
            ProtocolVersion::V1_16_3 => 753,
            ProtocolVersion::V1_16_2 => 751,
            ProtocolVersion::V1_16_1 => 736,
            ProtocolVersion::V1_16 => 735,
            ProtocolVersion::V1_15_2 => 578,
            ProtocolVersion::V1_15_1 => 575,
            ProtocolVersion::V1_15 => 573,
            ProtocolVersion::V1_14_4 => 498,
            ProtocolVersion::V1_14_3 => 490,
            ProtocolVersion::V1_14_2 => 485,
            ProtocolVersion::V1_14_1 => 480,
            ProtocolVersion::V1_14 => 477,
            ProtocolVersion::V1_13_2 => 404,
            ProtocolVersion::V1_13_1 => 401,
            ProtocolVersion::V1_13 => 393,
            ProtocolVersion::V1_12_2 => 340,
            ProtocolVersion::V1_12_1 => 338,
            ProtocolVersion::V1_12 => 335,
            ProtocolVersion::V1_11_1 => 316,
            ProtocolVersion::V1_11 => 315,
            ProtocolVersion::V1_10 => 210,
            ProtocolVersion::V1_9_3 => 110,
            ProtocolVersion::V1_9_2 => 109,
            ProtocolVersion::V1_9_1 => 108,
            ProtocolVersion::V1_9 => 107,
            ProtocolVersion::V1_8 => 47,
            ProtocolVersion::V1_7_6 => 5,
            ProtocolVersion::V1_7_2 => 4,
        }
    }

    pub fn version_name(&self) -> String {
        match self {
            ProtocolVersion::V1_21_4 => "1.21.4".to_string(),
            ProtocolVersion::V1_21_2 => "1.21.2".to_string(),
            ProtocolVersion::V1_21 => "1.21".to_string(),
            ProtocolVersion::V1_20_5 => "1.20.5".to_string(),
            ProtocolVersion::V1_20_3 => "1.20.3".to_string(),
            ProtocolVersion::V1_20_2 => "1.20.2".to_string(),
            ProtocolVersion::V1_20 => "1.20".to_string(),
            ProtocolVersion::V1_19_4 => "1.19.4".to_string(),
            ProtocolVersion::V1_19_3 => "1.19.3".to_string(),
            ProtocolVersion::V1_19_1 => "1.19.1".to_string(),
            ProtocolVersion::V1_19 => "1.19".to_string(),
            ProtocolVersion::V1_18_2 => "1.18.2".to_string(),
            ProtocolVersion::V1_18 => "1.18".to_string(),
            ProtocolVersion::V1_17_1 => "1.17.1".to_string(),
            ProtocolVersion::V1_17 => "1.17".to_string(),
            ProtocolVersion::V1_16_4 => "1.16.4".to_string(),
            ProtocolVersion::V1_16_3 => "1.16.3".to_string(),
            ProtocolVersion::V1_16_2 => "1.16.2".to_string(),
            ProtocolVersion::V1_16_1 => "1.16.1".to_string(),
            ProtocolVersion::V1_16 => "1.16".to_string(),
            ProtocolVersion::V1_15_2 => "1.15.2".to_string(),
            ProtocolVersion::V1_15_1 => "1.15.1".to_string(),
            ProtocolVersion::V1_15 => "1.15".to_string(),
            ProtocolVersion::V1_14_4 => "1.14.4".to_string(),
            ProtocolVersion::V1_14_3 => "1.14.3".to_string(),
            ProtocolVersion::V1_14_2 => "1.14.2".to_string(),
            ProtocolVersion::V1_14_1 => "1.14.1".to_string(),
            ProtocolVersion::V1_14 => "1.14".to_string(),
            ProtocolVersion::V1_13_2 => "1.13.2".to_string(),
            ProtocolVersion::V1_13_1 => "1.13.1".to_string(),
            ProtocolVersion::V1_13 => "1.13".to_string(),
            ProtocolVersion::V1_12_2 => "1.12.2".to_string(),
            ProtocolVersion::V1_12_1 => "1.12.1".to_string(),
            ProtocolVersion::V1_12 => "1.12".to_string(),
            ProtocolVersion::V1_11_1 => "1.11.1".to_string(),
            ProtocolVersion::V1_11 => "1.11".to_string(),
            ProtocolVersion::V1_10 => "1.10".to_string(),
            ProtocolVersion::V1_9_3 => "1.9.3".to_string(),
            ProtocolVersion::V1_9_2 => "1.9.2".to_string(),
            ProtocolVersion::V1_9_1 => "1.9.1".to_string(),
            ProtocolVersion::V1_9 => "1.9".to_string(),
            ProtocolVersion::V1_8 => "1.8".to_string(),
            ProtocolVersion::V1_7_6 => "1.7.6".to_string(),
            ProtocolVersion::V1_7_2 => "1.7.2".to_string(),
        }
    }
}

impl From<VarInt> for ProtocolVersion {
    fn from(value: VarInt) -> Self {
        match value.value() {
            769 => ProtocolVersion::V1_21_4,
            768 => ProtocolVersion::V1_21_2,
            767 => ProtocolVersion::V1_21,
            766 => ProtocolVersion::V1_20_5,
            765 => ProtocolVersion::V1_20_3,
            764 => ProtocolVersion::V1_20_2,
            763 => ProtocolVersion::V1_20,
            762 => ProtocolVersion::V1_19_4,
            761 => ProtocolVersion::V1_19_3,
            760 => ProtocolVersion::V1_19_1,
            759 => ProtocolVersion::V1_19,
            758 => ProtocolVersion::V1_18_2,
            757 => ProtocolVersion::V1_18,
            756 => ProtocolVersion::V1_17_1,
            755 => ProtocolVersion::V1_17,
            754 => ProtocolVersion::V1_16_4,
            753 => ProtocolVersion::V1_16_3,
            751 => ProtocolVersion::V1_16_2,
            736 => ProtocolVersion::V1_16_1,
            735 => ProtocolVersion::V1_16,
            578 => ProtocolVersion::V1_15_2,
            575 => ProtocolVersion::V1_15_1,
            573 => ProtocolVersion::V1_15,
            498 => ProtocolVersion::V1_14_4,
            490 => ProtocolVersion::V1_14_3,
            485 => ProtocolVersion::V1_14_2,
            480 => ProtocolVersion::V1_14_1,
            477 => ProtocolVersion::V1_14,
            404 => ProtocolVersion::V1_13_2,
            401 => ProtocolVersion::V1_13_1,
            393 => ProtocolVersion::V1_13,
            340 => ProtocolVersion::V1_12_2,
            338 => ProtocolVersion::V1_12_1,
            335 => ProtocolVersion::V1_12,
            316 => ProtocolVersion::V1_11_1,
            315 => ProtocolVersion::V1_11,
            210 => ProtocolVersion::V1_10,
            110 => ProtocolVersion::V1_9_3,
            109 => ProtocolVersion::V1_9_2,
            108 => ProtocolVersion::V1_9_1,
            107 => ProtocolVersion::V1_9,
            47 => ProtocolVersion::V1_8,
            5 => ProtocolVersion::V1_7_6,
            4 => ProtocolVersion::V1_7_2,
            // If the version is not recognized, default to the latest version in case it magically works
            _ => ProtocolVersion::default(),
        }
    }
}

impl PartialEq for ProtocolVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version_number() == other.version_number()
    }
}

impl Eq for ProtocolVersion {}

impl PartialOrd for ProtocolVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProtocolVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version_number().cmp(&other.version_number())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_ordering() {
        let v1_21 = ProtocolVersion::V1_21;
        let v1_21_2 = ProtocolVersion::V1_21_2;
        let v1_21_4 = ProtocolVersion::V1_21_4;

        assert!(v1_21 < v1_21_2);
        assert!(v1_21_2 < v1_21_4);
        assert!(v1_21_4 > v1_21_2);
        assert_eq!(v1_21_4, v1_21_4);
        assert_ne!(v1_21_2, v1_21_4);
    }
}
