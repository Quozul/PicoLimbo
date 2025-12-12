use crate::play::data::login_packet_data::post_v1_16::PostV1_16Data;
use crate::play::data::login_packet_data::post_v1_20_2::PostV1_20_2Data;
use crate::play::data::login_packet_data::pre_v1_16::{DimensionField, PreV1_16Data};
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct LoginPacket {
    /// The player's Entity ID (EID).
    entity_id: i32,
    data: LoginPacketData,
}

enum LoginPacketData {
    PreV1_16(PreV1_16Data),
    PostV1_16(PostV1_16Data),
    PostV1_20_2(PostV1_20_2Data),
}

impl EncodePacket for LoginPacketData {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            LoginPacketData::PreV1_16(value) => value.encode(writer, protocol_version),
            LoginPacketData::PostV1_16(value) => value.encode(writer, protocol_version),
            LoginPacketData::PostV1_20_2(value) => value.encode(writer, protocol_version),
        }
    }
}

impl LoginPacket {
    /// This is the constructor for version 1.16.2 up to 1.18.2 included
    pub fn with_dimension_codec(
        dimension: Dimension,
        registry_codec_bytes: &'static [u8],
        dimension_codec_bytes: &'static [u8],
    ) -> Self {
        let iden = dimension.identifier();
        Self {
            entity_id: 0,
            data: LoginPacketData::PostV1_16(PostV1_16Data {
                dimension_names: LengthPaddedVec::new(vec![iden.clone()]),
                world_name: iden.clone(),
                v1_19_dimension_type: iden.clone(),
                registry_codec_bytes: Omitted::Some(registry_codec_bytes),
                v1_16_2_dimension_codec_bytes: Omitted::Some(dimension_codec_bytes),
                ..PostV1_16Data::default()
            }),
        }
    }

    /// This is the constructor for 1.16, 1.16.1 and 1.19 up to 1.20 included
    pub fn with_registry_codec(dimension: Dimension, registry_codec_bytes: &'static [u8]) -> Self {
        let iden = dimension.identifier();
        Self {
            entity_id: 0,
            data: LoginPacketData::PostV1_16(PostV1_16Data {
                dimension_names: LengthPaddedVec::new(vec![iden.clone()]),
                world_name: iden.clone(),
                dimension_name: iden.clone(),
                registry_codec_bytes: Omitted::Some(registry_codec_bytes),
                v1_19_dimension_type: iden.clone(),
                ..PostV1_16Data::default()
            }),
        }
    }

    /// This is the constructor for all versions from 1.20.2 to 1.20.4 included
    pub fn with_dimension_post_v1_20_2(dimension: Dimension) -> Self {
        let iden = dimension.identifier();
        Self {
            entity_id: 0,
            data: LoginPacketData::PostV1_20_2(PostV1_20_2Data {
                dimension_names: LengthPaddedVec::new(vec![iden.clone()]),
                dimension_name: iden.clone(),
                dimension_type: iden.clone(),
                ..PostV1_20_2Data::default()
            }),
        }
    }

    /// This is the constructor for all versions from 1.7.2 to 1.15.2 included
    pub fn with_dimension_pre_v1_16(dimension: Dimension) -> Self {
        Self {
            entity_id: 0,
            data: LoginPacketData::PreV1_16(PreV1_16Data {
                dimension: DimensionField(dimension.legacy_i8()),
                ..PreV1_16Data::default()
            }),
        }
    }

    /// This is the constructor for all versions starting 1.20.5
    pub fn with_dimension_index(dimension: Dimension, dimension_index: i32) -> Self {
        let iden = dimension.identifier();
        Self {
            entity_id: 0,
            data: LoginPacketData::PostV1_20_2(PostV1_20_2Data {
                dimension_names: LengthPaddedVec::new(vec![iden.clone()]),
                dimension_name: iden.clone(),
                v1_20_5_dimension_type: dimension_index.into(),
                ..PostV1_20_2Data::default()
            }),
        }
    }

    pub fn set_game_mode(
        mut self,
        protocol_version: ProtocolVersion,
        game_mode: u8,
        is_hard_core: bool,
    ) -> Self {
        match &mut self.data {
            LoginPacketData::PreV1_16(value) => {
                if is_hard_core {
                    value.game_mode = game_mode | 0x8;
                } else {
                    value.game_mode = game_mode;
                }
            }
            LoginPacketData::PostV1_16(value) => {
                if is_hard_core && protocol_version.is_before_inclusive(ProtocolVersion::V1_16_1) {
                    value.game_mode = game_mode | 0x8;
                } else {
                    value.game_mode = game_mode;
                }
                value.v1_16_2_is_hardcore = is_hard_core;
            }
            LoginPacketData::PostV1_20_2(value) => {
                value.game_mode = game_mode;
                value.is_hardcore = is_hard_core;
            }
        }
        self
    }

    pub fn set_view_distance(mut self, view_distance: i32) -> Self {
        match &mut self.data {
            LoginPacketData::PreV1_16(value) => {
                value.v1_14_view_distance = view_distance.into();
            }
            LoginPacketData::PostV1_16(value) => {
                value.view_distance = view_distance.into();
                value.v1_18_simulation_distance = view_distance.into();
            }
            LoginPacketData::PostV1_20_2(value) => {
                value.view_distance = view_distance.into();
                value.simulation_distance = view_distance.into();
            }
        }
        self
    }

    pub fn set_reduced_debug_info(mut self, reduced_debug_info: bool) -> Self {
        match &mut self.data {
            LoginPacketData::PreV1_16(value) => {
                value.v1_8_reduced_debug_info = reduced_debug_info;
            }
            LoginPacketData::PostV1_16(value) => {
                value.reduced_debug_info = reduced_debug_info;
            }
            LoginPacketData::PostV1_20_2(value) => {
                value.reduced_debug_info = reduced_debug_info;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn expected_snapshots() -> HashMap<i32, Vec<u8>> {
        HashMap::from([
            (
                769,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 63, 1,
                ],
            ),
            (
                768,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 63, 1,
                ],
            ),
            (
                767,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 1,
                ],
            ),
            (
                766,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 1,
                ],
            ),
            (
                765,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 19, 109, 105, 110, 101,
                    99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0,
                ],
            ),
            (
                764,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 19, 109, 105, 110, 101,
                    99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0,
                ],
            ),
            (
                763,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0, 0,
                ],
            ),
            (
                762,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                761,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                760,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                759,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                758,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0,
                    1,
                ],
            ),
            (
                757,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0,
                    1,
                ],
            ),
            (
                756,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                755,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                754,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                753,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                751,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                736,
                vec![
                    0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5,
                    87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                735,
                vec![
                    0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5,
                    87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                578,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97,
                    117, 108, 116, 10, 0, 1,
                ],
            ),
            (
                575,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97,
                    117, 108, 116, 10, 0, 1,
                ],
            ),
            (
                573,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97,
                    117, 108, 116, 10, 0, 1,
                ],
            ),
            (
                498,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                490,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                485,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                480,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                477,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                404,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                401,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                393,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                340,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                338,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                335,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                316,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                315,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                210,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                110,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                109,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                108,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                107,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                47,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                5,
                vec![0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116],
            ),
            (
                4,
                vec![0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116],
            ),
        ])
    }

    static NBT_BYTES: &[u8] = &[
        8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100,
    ];

    fn create_packet(protocol_version: ProtocolVersion) -> LoginPacket {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_2) {
            LoginPacket {
                entity_id: 0,
                data: LoginPacketData::PostV1_20_2(PostV1_20_2Data::default()),
            }
        } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_16) {
            LoginPacket {
                entity_id: 0,
                data: LoginPacketData::PostV1_16(PostV1_16Data {
                    registry_codec_bytes: Omitted::Some(NBT_BYTES),
                    v1_16_2_dimension_codec_bytes: Omitted::Some(NBT_BYTES),
                    ..PostV1_16Data::default()
                }),
            }
        } else {
            LoginPacket {
                entity_id: 0,
                data: LoginPacketData::PreV1_16(PreV1_16Data::default()),
            }
        }
    }

    #[test]
    fn login_packet() {
        let snapshots = expected_snapshots();

        for (version, expected_bytes) in snapshots {
            let protocol_version = ProtocolVersion::from(version);
            let packet = create_packet(protocol_version);
            let mut writer = BinaryWriter::new();
            packet.encode(&mut writer, protocol_version).unwrap();
            let bytes = writer.into_inner();
            assert_eq!(bytes, expected_bytes, "Mismatch for version {}", version);
        }
    }
}
