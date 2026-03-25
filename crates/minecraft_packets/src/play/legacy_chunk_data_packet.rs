use blocks_report::{BlocksReportId, InternalId};
use minecraft_protocol::prelude::*;
use pico_nbt::Value;
use pico_structures::prelude::{Palette, World};

const SECTION_VOLUME: usize = 4096;
const LEGACY_SECTION_COUNT: usize = 16;
const LEGACY_BIOME_COUNT: usize = 256;
const LEGACY_LIGHT_ARRAY_SIZE: usize = 2048;
const LEGACY_AIR_STATE_ID: u16 = 0;
const LEGACY_FALLBACK_STONE_STATE_ID: u16 = 0x10; // stone
const LEGACY_GLOBAL_BITS_PER_BLOCK: u8 = 13;

#[derive(PacketOut)]
pub struct LegacyChunkDataPacket {
    chunk_x: i32,
    chunk_z: i32,
    full_chunk: bool,
    #[pvn(..48)]
    v1_8_primary_bit_mask: u16,
    #[pvn(107..)]
    v1_9_primary_bit_mask: VarInt,
    data: LengthPaddedVec<u8>,
    #[pvn(107..)]
    block_entities: LengthPaddedVec<Value>,
}

impl LegacyChunkDataPacket {
    pub fn void(chunk_x: i32, chunk_z: i32, biome_id: i32) -> Self {
        let biome = u8::try_from(biome_id).unwrap_or(1);
        let biome_data = vec![biome; LEGACY_BIOME_COUNT];

        Self {
            chunk_x,
            chunk_z,
            full_chunk: true,
            v1_8_primary_bit_mask: 0,
            v1_9_primary_bit_mask: VarInt::new(0),
            data: LengthPaddedVec::new(biome_data),
            block_entities: LengthPaddedVec::default(),
        }
    }

    pub fn from_structure(
        chunk_x: i32,
        chunk_z: i32,
        biome_id: i32,
        world: &World,
        air_internal_id: InternalId,
        report_id_mapping: &[BlocksReportId],
        protocol_version: ProtocolVersion,
        has_sky_light: bool,
    ) -> Self {
        let mut included_sections: Vec<(usize, Vec<u16>)> = Vec::new();
        let mut primary_bit_mask: u16 = 0;

        for section_y in 0..LEGACY_SECTION_COUNT {
            let coordinates = Coordinates::new(chunk_x, section_y as i32, chunk_z);
            let Some(section_palette) = world.get_section(&coordinates) else {
                continue;
            };

            let section_states =
                section_states_from_palette(section_palette, air_internal_id, report_id_mapping);
            if section_states
                .iter()
                .all(|&state_id| state_id == LEGACY_AIR_STATE_ID)
            {
                continue;
            }

            primary_bit_mask |= 1 << section_y;
            included_sections.push((section_y, section_states));
        }

        if included_sections.is_empty() {
            return Self::void(chunk_x, chunk_z, biome_id);
        }

        let block_light_sections = world.get_chunk_block_light(chunk_x, chunk_z);
        let sky_light_sections = world.get_chunk_sky_light(chunk_x, chunk_z);
        let biome = u8::try_from(biome_id).unwrap_or(1);
        let biome_data = vec![biome; LEGACY_BIOME_COUNT];

        let mut data = Vec::new();

        if protocol_version == ProtocolVersion::V1_8 {
            for (_, section_states) in &included_sections {
                append_section_blocks_v1_8(&mut data, section_states);
            }

            for (section_y, _) in &included_sections {
                append_light_section(&mut data, block_light_sections, *section_y, 0x00);
            }

            if has_sky_light {
                for (section_y, _) in &included_sections {
                    append_light_section(&mut data, sky_light_sections, *section_y, 0xFF);
                }
            }
        } else {
            for (section_y, section_states) in &included_sections {
                append_section_blocks_v1_9_to_v1_12(&mut data, section_states);
                append_light_section(&mut data, block_light_sections, *section_y, 0x00);
                if has_sky_light {
                    append_light_section(&mut data, sky_light_sections, *section_y, 0xFF);
                }
            }
        }

        data.extend_from_slice(&biome_data);

        Self {
            chunk_x,
            chunk_z,
            full_chunk: true,
            v1_8_primary_bit_mask: primary_bit_mask,
            v1_9_primary_bit_mask: VarInt::new(i32::from(primary_bit_mask)),
            data: LengthPaddedVec::new(data),
            block_entities: LengthPaddedVec::default(),
        }
    }
}

fn section_states_from_palette(
    palette: &Palette,
    air_internal_id: InternalId,
    report_id_mapping: &[BlocksReportId],
) -> Vec<u16> {
    match palette {
        Palette::Single { internal_id } => {
            vec![
                map_internal_state_to_legacy(*internal_id, air_internal_id, report_id_mapping);
                SECTION_VOLUME
            ]
        }
        Palette::Direct { internal_data } => internal_data
            .iter()
            .map(|internal_id| {
                map_internal_state_to_legacy(*internal_id, air_internal_id, report_id_mapping)
            })
            .collect(),
        Palette::Paletted {
            bits_per_entry,
            internal_palette,
            packed_data,
        } => {
            let bits_per_entry = usize::from(*bits_per_entry);
            if bits_per_entry == 0 {
                return vec![LEGACY_AIR_STATE_ID; SECTION_VOLUME];
            }

            let entries_per_long = 64 / bits_per_entry;
            let bit_mask = (1u64 << bits_per_entry) - 1;
            let mapped_palette: Vec<u16> = internal_palette
                .iter()
                .map(|internal_id| {
                    map_internal_state_to_legacy(*internal_id, air_internal_id, report_id_mapping)
                })
                .collect();

            let mut section_states = vec![LEGACY_AIR_STATE_ID; SECTION_VOLUME];
            for (index, entry) in section_states.iter_mut().enumerate() {
                let long_index = index / entries_per_long;
                let bit_offset = (index % entries_per_long) * bits_per_entry;
                let palette_index = packed_data
                    .get(long_index)
                    .map(|packed| ((packed >> bit_offset) & bit_mask) as usize)
                    .unwrap_or(0);

                *entry = mapped_palette
                    .get(palette_index)
                    .copied()
                    .unwrap_or(LEGACY_AIR_STATE_ID);
            }
            section_states
        }
    }
}

fn map_internal_state_to_legacy(
    internal_id: InternalId,
    air_internal_id: InternalId,
    report_id_mapping: &[BlocksReportId],
) -> u16 {
    if internal_id == air_internal_id {
        return LEGACY_AIR_STATE_ID;
    }

    report_id_mapping
        .get(internal_id as usize)
        .copied()
        .unwrap_or(LEGACY_FALLBACK_STONE_STATE_ID)
}

fn append_section_blocks_v1_8(data: &mut Vec<u8>, section_states: &[u16]) {
    for state_id in section_states {
        data.push((state_id & 0x00FF) as u8);
        data.push((state_id >> 8) as u8);
    }
}

fn append_section_blocks_v1_9_to_v1_12(data: &mut Vec<u8>, section_states: &[u16]) {
    data.push(LEGACY_GLOBAL_BITS_PER_BLOCK);
    let packed_data = pack_legacy_compact_array(section_states, LEGACY_GLOBAL_BITS_PER_BLOCK);
    append_varint(data, packed_data.len() as i32);

    for word in packed_data {
        data.extend_from_slice(&word.to_be_bytes());
    }
}

fn pack_legacy_compact_array(section_states: &[u16], bits_per_block: u8) -> Vec<u64> {
    let bits_per_block = usize::from(bits_per_block);
    let total_bits = section_states.len() * bits_per_block;
    let array_length = total_bits.div_ceil(64);
    let mut packed = vec![0u64; array_length];
    let mask = (1u64 << bits_per_block) - 1;

    for (index, state_id) in section_states.iter().enumerate() {
        let value = u64::from(*state_id) & mask;
        let bit_index = index * bits_per_block;
        let word_index = bit_index / 64;
        let bit_offset = bit_index % 64;

        packed[word_index] |= value << bit_offset;
        let spill = bit_offset + bits_per_block;
        if spill > 64 && word_index + 1 < packed.len() {
            packed[word_index + 1] |= value >> (64 - bit_offset);
        }
    }
    packed
}

fn append_light_section(
    data: &mut Vec<u8>,
    light_sections: Option<&[pico_structures::prelude::LightSection]>,
    section_y: usize,
    default_value: u8,
) {
    if let Some(light_section) = light_sections.and_then(|sections| sections.get(section_y)) {
        data.extend(light_section.iter().map(|byte| *byte as u8));
    } else {
        let new_len = data.len() + LEGACY_LIGHT_ARRAY_SIZE;
        data.resize(new_len, default_value);
    }
}

fn append_varint(data: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        if (value & !0x7F) == 0 {
            data.push(value as u8);
            return;
        }
        data.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn legacy_state(block_id: u16, metadata: u8) -> u16 {
        (block_id << 4) | ((metadata as u16) & 0x0F)
    }

    #[test]
    fn legacy_chunk_packet_v1_8_layout() {
        let packet = LegacyChunkDataPacket::void(0, 0, 1);
        let mut writer = BinaryWriter::new();
        packet
            .encode(&mut writer, ProtocolVersion::V1_8)
            .expect("encode");
        let bytes = writer.into_inner();

        assert_eq!(bytes.len(), 269);
        assert_eq!(&bytes[0..9], &[0, 0, 0, 0, 0, 0, 0, 0, 1]); // x, z, full_chunk
        assert_eq!(&bytes[9..11], &[0, 0]); // short bitmask in 1.8
        assert_eq!(&bytes[11..13], &[0x80, 0x02]); // VarInt data length = 256
    }

    #[test]
    fn legacy_chunk_packet_v1_9_layout() {
        let packet = LegacyChunkDataPacket::void(0, 0, 1);
        let mut writer = BinaryWriter::new();
        packet
            .encode(&mut writer, ProtocolVersion::V1_9)
            .expect("encode");
        let bytes = writer.into_inner();

        assert_eq!(bytes.len(), 269);
        assert_eq!(&bytes[0..9], &[0, 0, 0, 0, 0, 0, 0, 0, 1]); // x, z, full_chunk
        assert_eq!(bytes[9], 0); // VarInt bitmask in 1.9+
        assert_eq!(&bytes[10..12], &[0x80, 0x02]); // VarInt data length = 256
        assert_eq!(bytes[268], 0); // empty block entities length
    }

    #[test]
    fn legacy_v1_8_section_state_encoding() {
        let mut section = vec![LEGACY_AIR_STATE_ID; SECTION_VOLUME];
        section[0] = legacy_state(5, 0);
        section[1] = legacy_state(35, 14);

        let mut encoded = Vec::new();
        append_section_blocks_v1_8(&mut encoded, &section);

        assert_eq!(encoded.len(), 8192);
        assert_eq!(&encoded[0..4], &[0x50, 0x00, 0x3E, 0x02]);
        assert_eq!(&encoded[4..6], &[0x00, 0x00]);
    }

    #[test]
    fn legacy_v1_9_to_v1_12_compact_packing_roundtrip() {
        let mut input = vec![LEGACY_AIR_STATE_ID; SECTION_VOLUME];
        input[0] = legacy_state(1, 0);
        input[1] = legacy_state(5, 0);
        input[2] = legacy_state(35, 14);
        input[3] = legacy_state(95, 11);
        input[4095] = legacy_state(98, 1);

        let packed = pack_legacy_compact_array(&input, LEGACY_GLOBAL_BITS_PER_BLOCK);
        assert_eq!(packed.len(), 832);

        let decoded =
            unpack_legacy_compact_array(&packed, LEGACY_GLOBAL_BITS_PER_BLOCK, SECTION_VOLUME);
        assert_eq!(decoded, input);
    }

    #[test]
    fn legacy_internal_state_mapping_uses_report_mapping() {
        let report_mapping = vec![
            LEGACY_AIR_STATE_ID,
            legacy_state(5, 0),
            legacy_state(35, 14),
        ];
        let air_internal_id = 0;

        assert_eq!(
            map_internal_state_to_legacy(air_internal_id, air_internal_id, &report_mapping),
            LEGACY_AIR_STATE_ID
        );
        assert_eq!(
            map_internal_state_to_legacy(1, air_internal_id, &report_mapping),
            legacy_state(5, 0)
        );
        assert_eq!(
            map_internal_state_to_legacy(2, air_internal_id, &report_mapping),
            legacy_state(35, 14)
        );
        assert_eq!(
            map_internal_state_to_legacy(9999, air_internal_id, &report_mapping),
            LEGACY_FALLBACK_STONE_STATE_ID
        );
    }

    fn unpack_legacy_compact_array(
        packed: &[u64],
        bits_per_block: u8,
        value_count: usize,
    ) -> Vec<u16> {
        let bits_per_block = usize::from(bits_per_block);
        let mask = (1u64 << bits_per_block) - 1;
        let mut output = vec![0u16; value_count];

        for (index, entry) in output.iter_mut().enumerate() {
            let bit_index = index * bits_per_block;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;

            let mut value = (packed[word_index] >> bit_offset) & mask;
            let spill = bit_offset + bits_per_block;

            if spill > 64 && word_index + 1 < packed.len() {
                let carry_bits = spill - 64;
                let carry_mask = (1u64 << carry_bits) - 1;
                value |= (packed[word_index + 1] & carry_mask) << (bits_per_block - carry_bits);
            }

            *entry = value as u16;
        }

        output
    }
}
