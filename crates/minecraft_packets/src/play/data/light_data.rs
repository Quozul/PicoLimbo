use minecraft_protocol::prelude::{
    BitSet, EncodePacketField, LengthPaddedVec, LengthPaddedVecEncodeError, VarInt,
};
use thiserror::Error;

#[derive(Debug)]
pub struct LightData {
    sky_light_mask: BitSet,
    block_light_mask: BitSet,
    empty_sky_light_mask: BitSet,
    empty_block_light_mask: BitSet,
    sky_light_arrays: LengthPaddedVec<Light>,
    block_light_arrays: LengthPaddedVec<Light>,
}

#[derive(Debug, Clone)]
pub struct Light {
    /// Length of the following array is always 2048
    /// There is 1 array for each bit set to true in the light mask, starting with the lowest value. Half a byte per light value. Indexed ((y<<8) | (z<<4) | x) / 2 If there's a remainder, masked 0xF0 else 0x0F.
    block_light_array: LengthPaddedVec<i8>,
}

impl EncodePacketField for Light {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        let size = VarInt::new(self.block_light_array.len() as i32);
        size.encode(bytes, protocol_version)?;
        for &value in &self.block_light_array {
            bytes.push(value as u8);
        }
        Ok(())
    }
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            sky_light_mask: BitSet::default(),
            block_light_mask: BitSet::default(),
            empty_sky_light_mask: BitSet::default(),
            empty_block_light_mask: BitSet::default(),
            sky_light_arrays: Vec::new().into(),
            block_light_arrays: Vec::new().into(),
        }
    }
}

impl LightData {
    /// Creates LightData for a world with a uniform custom light level.
    /// A level of 0 will use the optimized "dark" path.
    ///
    /// # Panics
    /// Panics if `light_level` is greater than 15.
    pub fn new_with_level(light_level: u8) -> Self {
        // The world has 24 sections, from Y=-64 to Y=319.
        const NUM_SECTIONS_IN_WORLD: u32 = 24;

        if light_level == 0 {
            // If the light level is 0, we can use the optimized dark implementation.
            return Self::default();
        }

        assert!(light_level <= 15, "Light level must be between 0 and 15.");

        // --- 1. Create the mask for the 24 world sections ---
        // We need to set bits 1 through 24 (inclusive) in the mask.
        // Bit 0 is for the section below the world, bit 25 is for the one above.
        // `((1 << 24) - 1)` creates 24 set bits (0...0111...1).
        // `<< 1` shifts them to occupy bits 1 through 24.
        let world_sections_mask_val = ((1u64 << NUM_SECTIONS_IN_WORLD) - 1) << 1;

        let world_sections_mask = BitSet::new(vec![world_sections_mask_val as i64]);

        // --- 2. Create a single, uniformly lit light section array ---
        // Pack the two 4-bit light levels into a single i8.
        // e.g., level 15 (0xF) -> 0b11111111 -> 0xFF -> -1 as i8
        // e.g., level 7 (0x7) -> 0b01110111 -> 0x77
        let packed_byte = ((light_level << 4) | light_level) as i8;

        // A light section is always 2048 bytes. Fill it with our packed value.
        let light_section_array = Light {
            block_light_array: LengthPaddedVec::new(vec![packed_byte; 2048]),
        };

        // --- 3. Create the list of 24 light arrays for the chunk column ---
        let all_light_arrays =
            LengthPaddedVec::new(vec![light_section_array; NUM_SECTIONS_IN_WORLD as usize]);

        Self {
            // Set the masks to indicate that all 24 world sections have data.
            sky_light_mask: world_sections_mask.clone(),
            block_light_mask: world_sections_mask,

            // The empty masks must be clear, as we are providing data.
            empty_sky_light_mask: BitSet::default(),
            empty_block_light_mask: BitSet::default(),

            // Provide the actual light data.
            // Note: We use the same arrays for both sky and block light for simplicity.
            sky_light_arrays: all_light_arrays.clone(),
            block_light_arrays: all_light_arrays,
        }
    }
}

#[derive(Debug, Error)]
pub enum LightDataError {
    #[error(transparent)]
    LengthPaddedVecEncodeError(#[from] LengthPaddedVecEncodeError),
}

impl EncodePacketField for LightData {
    type Error = LightDataError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        self.sky_light_mask.encode(bytes, protocol_version)?;
        self.block_light_mask.encode(bytes, protocol_version)?;
        self.empty_sky_light_mask.encode(bytes, protocol_version)?;
        self.empty_block_light_mask
            .encode(bytes, protocol_version)?;
        self.sky_light_arrays.encode(bytes, protocol_version)?;
        self.block_light_arrays.encode(bytes, protocol_version)?;
        Ok(())
    }
}
