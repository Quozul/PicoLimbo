use macros::Pvn;
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Default, Clone, Debug, Pvn)]
pub enum ProtocolVersion {
    #[default]
    #[pvn(771, reports = "V1_21_5", data = "V1_21_5")]
    V1_21_6,
    #[pvn(770)]
    V1_21_5,
    #[pvn(769)]
    V1_21_4,
    #[pvn(768)]
    V1_21_2,
    #[pvn(767)]
    V1_21,

    #[pvn(766)]
    V1_20_5,
    #[pvn(765, data = "V1_20")]
    V1_20_3,
    #[pvn(764, data = "V1_20")]
    V1_20_2,
    #[pvn(763)]
    V1_20,

    #[pvn(762)]
    V1_19_4,
    #[pvn(761, data = "V1_19")]
    V1_19_3,
    #[pvn(760, data = "V1_19")]
    V1_19_1,
    #[pvn(759)]
    V1_19,

    /// [Minecraft 1.18.2](https://minecraft.wiki/w/Java_Edition_1.18.2) released on February 28, 2022.
    /// See the [protocol documentation version 758](https://minecraft.wiki/w/Java_Edition_protocol?oldid=2772783)
    #[pvn(758, reports = "V1_18")]
    V1_18_2,
    /// [Minecraft 1.18](https://minecraft.wiki/w/Java_Edition_1.18) released on November 30, 2021.
    /// See the [protocol documentation version 757](https://minecraft.wiki/w/Java_Edition_protocol?oldid=2772738)
    #[pvn(757)]
    V1_18,

    #[pvn(756, reports = "V1_17", data = "V1_17")]
    V1_17_1,
    #[pvn(755)]
    V1_17,

    #[pvn(754, reports = "V1_16_2", data = "V1_16_2")]
    V1_16_4,
    #[pvn(753, reports = "V1_16_2", data = "V1_16_2")]
    V1_16_3,
    /// Minecraft version 1.16.2 changes the format of the registries.
    #[pvn(751)]
    V1_16_2,
    /// Minecraft version 1.16.1 is basically the same as 1.16.
    #[pvn(736, reports = "V1_16", data = "V1_16")]
    V1_16_1,

    /// Minecraft version 1.16 introduced the first registries to be sent in the Join Game packet.
    /// Also, string-encoded UUID are now sent as a UUID in the Game Profile Packet.
    #[pvn(735)]
    V1_16,

    #[pvn(578, reports = "V1_15")]
    V1_15_2,
    #[pvn(575, reports = "V1_15")]
    V1_15_1,
    #[pvn(573)]
    V1_15,

    #[pvn(498, reports = "V1_14")]
    V1_14_4,
    #[pvn(490, reports = "V1_14")]
    V1_14_3,
    #[pvn(485, reports = "V1_14")]
    V1_14_2,
    #[pvn(480, reports = "V1_14")]
    V1_14_1,
    #[pvn(477)]
    V1_14,

    #[pvn(404, reports = "V1_13")]
    V1_13_2,
    #[pvn(401, reports = "V1_13")]
    V1_13_1,
    #[pvn(393)]
    V1_13,

    #[pvn(340, reports = "V1_12_1")]
    V1_12_2,
    #[pvn(338)]
    V1_12_1,
    #[pvn(335)]
    V1_12,

    #[pvn(316, reports = "V1_11")]
    V1_11_1,
    #[pvn(315)]
    V1_11,

    #[pvn(210)]
    V1_10,

    #[pvn(110, reports = "V1_9")]
    V1_9_3,
    #[pvn(109, reports = "V1_9")]
    V1_9_2,
    #[pvn(108, reports = "V1_9")]
    V1_9_1,
    #[pvn(107)]
    V1_9,

    #[pvn(47)]
    V1_8,

    #[pvn(5, reports = "V1_7_2")]
    V1_7_6,
    #[pvn(4)]
    V1_7_2,
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

impl ProtocolVersion {
    pub fn between_inclusive(&self, min_version: Self, max_version: Self) -> bool {
        self >= &min_version && self <= &max_version
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
