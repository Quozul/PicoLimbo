use minecraft_protocol::prelude::*;

#[derive(PacketOut, Default)]
pub struct LightData {
    sky_light_mask: BitSet,
    block_light_mask: BitSet,
    empty_sky_light_mask: BitSet,
    empty_block_light_mask: BitSet,
    sky_light_arrays: LengthPaddedVec<Light>,
    block_light_arrays: LengthPaddedVec<Light>,
}

#[derive(PacketOut, Default, Clone)]
pub struct Light {
    /// Length of the following array is always 2048
    /// There is 1 array for each bit set to true in the light mask, starting with the lowest value. Half a byte per light value. Indexed ((y<<8) | (z<<4) | x) / 2 If there's a remainder, masked 0xF0 else 0x0F.
    block_light_array: LengthPaddedVec<i8>,
}

impl LightData {
    pub fn new_with_level(light_level: u8) -> Self {
        const NUM_SECTIONS_IN_WORLD: u32 = 24;
        let light_level = light_level.clamp(0, 15);

        if light_level == 0 {
            return Self::default();
        }

        let world_sections_mask_val = ((1u64 << NUM_SECTIONS_IN_WORLD) - 1) << 1;

        let world_sections_mask = BitSet::new(vec![world_sections_mask_val as i64]);

        let packed_byte = ((light_level << 4) | light_level) as i8;

        let light_section_array = Light {
            block_light_array: LengthPaddedVec::new(vec![packed_byte; 2048]),
        };

        let all_light_arrays =
            LengthPaddedVec::new(vec![light_section_array; NUM_SECTIONS_IN_WORLD as usize]);

        Self {
            sky_light_mask: world_sections_mask.clone(),
            block_light_mask: world_sections_mask,

            empty_sky_light_mask: BitSet::default(),
            empty_block_light_mask: BitSet::default(),

            sky_light_arrays: all_light_arrays.clone(),
            block_light_arrays: all_light_arrays,
        }
    }
}
