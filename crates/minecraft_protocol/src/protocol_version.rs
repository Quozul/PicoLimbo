use macros::Pvn;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Pvn)]
#[repr(i32)]
pub enum ProtocolVersion {
    #[default]
    #[pvn(reports = "V1_21_5", data = "V1_21_5")]
    V1_21_7 = 772,
    #[pvn(reports = "V1_21_5", data = "V1_21_5")]
    V1_21_6 = 771,
    V1_21_5 = 770,
    V1_21_4 = 769,
    V1_21_2 = 768,
    V1_21 = 767,

    V1_20_5 = 766,
    #[pvn(data = "V1_20")]
    V1_20_3 = 765,
    #[pvn(data = "V1_20")]
    V1_20_2 = 764,
    V1_20 = 763,

    V1_19_4 = 762,
    #[pvn(data = "V1_19")]
    V1_19_3 = 761,
    #[pvn(data = "V1_19")]
    V1_19_1 = 760,
    V1_19 = 759,

    #[pvn(reports = "V1_18")]
    V1_18_2 = 758,
    V1_18 = 757,

    #[pvn(reports = "V1_17", data = "V1_17")]
    V1_17_1 = 756,
    V1_17 = 755,

    #[pvn(reports = "V1_16_2", data = "V1_16_2")]
    V1_16_4 = 754,
    #[pvn(reports = "V1_16_2", data = "V1_16_2")]
    V1_16_3 = 753,
    V1_16_2 = 751,
    #[pvn(reports = "V1_16", data = "V1_16")]
    V1_16_1 = 736,
    V1_16 = 735,

    #[pvn(reports = "V1_15")]
    V1_15_2 = 578,
    #[pvn(reports = "V1_15")]
    V1_15_1 = 575,
    V1_15 = 573,

    #[pvn(reports = "V1_14")]
    V1_14_4 = 498,
    #[pvn(reports = "V1_14")]
    V1_14_3 = 490,
    #[pvn(reports = "V1_14")]
    V1_14_2 = 485,
    #[pvn(reports = "V1_14")]
    V1_14_1 = 480,
    V1_14 = 477,

    #[pvn(reports = "V1_13")]
    V1_13_2 = 404,
    #[pvn(reports = "V1_13")]
    V1_13_1 = 401,
    V1_13 = 393,

    #[pvn(reports = "V1_12_1")]
    V1_12_2 = 340,
    V1_12_1 = 338,
    V1_12 = 335,

    #[pvn(reports = "V1_11")]
    V1_11_1 = 316,
    V1_11 = 315,

    V1_10 = 210,

    #[pvn(reports = "V1_9")]
    V1_9_3 = 110,
    #[pvn(reports = "V1_9")]
    V1_9_2 = 109,
    #[pvn(reports = "V1_9")]
    V1_9_1 = 108,
    V1_9 = 107,

    V1_8 = 47,

    #[pvn(reports = "V1_7_2")]
    V1_7_6 = 5,
    V1_7_2 = 4,

    /// A special value to represent any protocol version.
    #[pvn(reports = "V1_7_2")]
    Any = -1,
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
