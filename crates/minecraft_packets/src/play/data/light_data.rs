use minecraft_protocol::prelude::*;
use pico_structures::prelude::LightSection;

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

impl Light {
    pub fn new(data: Vec<i8>) -> Self {
        Self {
            block_light_array: LengthPaddedVec::new(data),
        }
    }
}

impl LightData {
    pub fn from_light_data(
        sky_light_sections: &[LightSection],
        block_light_sections: &[LightSection],
        dimension_height: i32,
    ) -> Self {
        let world_section_count = dimension_height / 16;
        let total_light_sections = (world_section_count + 2) as u32;

        let all_sections_mask_val = (1u64 << total_light_sections) - 1;
        let all_sections_mask = BitSet::new(vec![all_sections_mask_val as i64]);

        let mut sky_light_arrays = Vec::with_capacity(total_light_sections as usize);
        sky_light_arrays.push(Light::new(vec![0xFFu8 as i8; 2048]));
        for section in sky_light_sections {
            sky_light_arrays.push(Light::new(section.clone()));
        }
        sky_light_arrays.push(Light::new(vec![0xFFu8 as i8; 2048]));
        while sky_light_arrays.len() < total_light_sections as usize {
            sky_light_arrays.push(Light::new(vec![0xFFu8 as i8; 2048]));
        }

        let mut block_light_arrays = Vec::with_capacity(total_light_sections as usize);
        block_light_arrays.push(Light::new(vec![0i8; 2048]));
        for section in block_light_sections {
            block_light_arrays.push(Light::new(section.clone()));
        }
        block_light_arrays.push(Light::new(vec![0i8; 2048]));
        while block_light_arrays.len() < total_light_sections as usize {
            block_light_arrays.push(Light::new(vec![0i8; 2048]));
        }

        Self {
            sky_light_mask: all_sections_mask.clone(),
            block_light_mask: all_sections_mask.clone(),
            empty_sky_light_mask: BitSet::default(),
            empty_block_light_mask: BitSet::default(),
            sky_light_arrays: LengthPaddedVec::new(sky_light_arrays),
            block_light_arrays: LengthPaddedVec::new(block_light_arrays),
        }
    }

    pub fn new_void(dimension_height: i32) -> Self {
        let world_section_count = dimension_height / 16;
        let total_light_sections = (world_section_count + 2) as u32;

        let all_sections_mask_val = (1u64 << total_light_sections) - 1;
        let all_sections_mask = BitSet::new(vec![all_sections_mask_val as i64]);

        let full_sky_light = Light::new(vec![0xFFu8 as i8; 2048]);
        let no_block_light = Light::new(vec![0i8; 2048]);

        Self {
            sky_light_mask: all_sections_mask.clone(),
            block_light_mask: all_sections_mask.clone(),
            empty_sky_light_mask: BitSet::default(),
            empty_block_light_mask: BitSet::default(),
            sky_light_arrays: LengthPaddedVec::new(vec![
                full_sky_light;
                total_light_sections as usize
            ]),
            block_light_arrays: LengthPaddedVec::new(vec![
                no_block_light;
                total_light_sections as usize
            ]),
        }
    }
}
