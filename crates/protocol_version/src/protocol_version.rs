use protocol_version_macro::Pvn;
use std::cmp::PartialEq;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Pvn)]
#[repr(i32)]
pub enum ProtocolVersion {
    #[pvn(data = V1_21_6)]
    #[default]
    V1_21_9 = 773,
    #[pvn(reports = V1_21_6, data = V1_21_6)]
    V1_21_7 = 772,
    V1_21_6 = 771,
    V1_21_5 = 770,
    V1_21_4 = 769,
    /// Docs: https://minecraft.wiki/w/Java_Edition_protocol/Packets?oldid=2780220
    V1_21_2 = 768,
    V1_21 = 767,

    V1_20_5 = 766,
    #[pvn(data = V1_20_2)]
    V1_20_3 = 765,
    V1_20_2 = 764,
    V1_20 = 763,

    V1_19_4 = 762,
    #[pvn(data = V1_19)]
    V1_19_3 = 761,
    #[pvn(data = V1_19)]
    V1_19_1 = 760,
    V1_19 = 759,

    #[pvn(reports = V1_18)]
    V1_18_2 = 758,
    V1_18 = 757,

    #[pvn(reports = V1_17, data = V1_17)]
    V1_17_1 = 756,
    V1_17 = 755,

    #[pvn(reports = V1_16_2, data = V1_16_2)]
    V1_16_4 = 754,
    #[pvn(reports = V1_16_2, data = V1_16_2)]
    V1_16_3 = 753,
    V1_16_2 = 751,
    #[pvn(reports = V1_16, data = V1_16)]
    V1_16_1 = 736,
    V1_16 = 735,

    #[pvn(reports = V1_15)]
    V1_15_2 = 578,
    #[pvn(reports = V1_15)]
    V1_15_1 = 575,
    V1_15 = 573,

    #[pvn(reports = V1_14)]
    V1_14_4 = 498,
    #[pvn(reports = V1_14)]
    V1_14_3 = 490,
    #[pvn(reports = V1_14)]
    V1_14_2 = 485,
    #[pvn(reports = V1_14)]
    V1_14_1 = 480,
    V1_14 = 477,

    #[pvn(reports = V1_13)]
    V1_13_2 = 404,
    #[pvn(reports = V1_13)]
    V1_13_1 = 401,
    V1_13 = 393,

    #[pvn(reports = V1_12_1)]
    V1_12_2 = 340,
    V1_12_1 = 338,
    V1_12 = 335,

    #[pvn(reports = V1_11)]
    V1_11_1 = 316,
    V1_11 = 315,

    V1_10 = 210,

    V1_9_3 = 110,
    #[pvn(reports = V1_9)]
    V1_9_2 = 109,
    #[pvn(reports = V1_9)]
    V1_9_1 = 108,
    V1_9 = 107,

    V1_8 = 47,

    #[pvn(reports = V1_7_2)]
    V1_7_6 = 5,
    V1_7_2 = 4,

    /// A special value to represent any protocol version.
    Any = -1,

    /// A special value to represent an unknown protocol version.
    #[pvn(reports = Any)]
    Unsupported = -2,
}

impl ProtocolVersion {
    #[inline]
    pub fn between_inclusive(&self, min_version: Self, max_version: Self) -> bool {
        self >= &min_version && self <= &max_version
    }

    #[inline]
    pub fn is_after_inclusive(&self, other: Self) -> bool {
        self >= &other
    }

    #[inline]
    pub fn is_before_inclusive(&self, max_version: Self) -> bool {
        self <= &max_version
    }

    #[inline]
    pub fn supports_configuration_state(&self) -> bool {
        self.is_after_inclusive(ProtocolVersion::V1_20_2)
    }

    #[inline]
    pub fn is_modern(&self) -> bool {
        self.is_after_inclusive(ProtocolVersion::V1_13)
    }

    #[inline]
    pub fn is_any(&self) -> bool {
        *self == Self::Any
    }

    #[inline]
    pub fn is_unsupported(&self) -> bool {
        *self == Self::Unsupported
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

    #[test]
    fn test_reports() {
        let v1_7_6 = ProtocolVersion::V1_7_6;
        let v1_7_2 = ProtocolVersion::V1_7_2;

        assert_eq!(v1_7_6.reports(), v1_7_2);
        assert_eq!(v1_7_2.reports(), v1_7_2);
    }

    #[test]
    fn oldest() {
        let oldest = ProtocolVersion::oldest();
        let v1_7_2 = ProtocolVersion::V1_7_2;

        assert_eq!(oldest, v1_7_2);
    }
}
