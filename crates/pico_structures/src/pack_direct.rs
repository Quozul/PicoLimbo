/// This function only works for 1.16 and after:
/// In prior versions, entries could cross long boundaries, and there was no padding.
pub fn pack_direct(mut entries_iter: impl Iterator<Item = u32>, bits_per_entry: u8) -> Vec<u64> {
    assert!(
        bits_per_entry > 0 && bits_per_entry <= 32,
        "bits_per_entry must be between 1 and 32"
    );
    let bpe = bits_per_entry as usize;
    let epl = 64 / bpe;
    assert!(epl > 0, "bits_per_entry cannot be greater than 64");

    let mask = (1u64 << bits_per_entry) - 1;

    let capacity = (4096 / epl) + 1;
    let mut packed_data = Vec::with_capacity(capacity);

    'outer: loop {
        let mut word = 0u64;
        for j in 0..epl {
            if let Some(id) = entries_iter.next() {
                let shift = (j * bpe) as u32;
                word |= ((id as u64) & mask) << shift;
            } else {
                if j > 0 {
                    packed_data.push(word);
                }
                break 'outer;
            }
        }
        packed_data.push(word);
    }

    packed_data
}

/// Packs entries in the compact format used before 1.16, where entries may cross long boundaries.
pub fn pack_compact(entries_iter: impl Iterator<Item = u32>, bits_per_entry: u8) -> Vec<u64> {
    assert!(
        bits_per_entry > 0 && bits_per_entry <= 32,
        "bits_per_entry must be between 1 and 32"
    );

    let bpe = bits_per_entry as usize;
    let mask = (1u64 << bits_per_entry) - 1;
    let (lower_bound, upper_bound) = entries_iter.size_hint();
    let estimated_entries = upper_bound.unwrap_or(lower_bound);
    let capacity = estimated_entries
        .checked_mul(bpe)
        .and_then(|bits| bits.checked_add(63))
        .map_or(0, |bits| bits / 64);
    let mut packed_data = Vec::with_capacity(capacity);
    let mut word = 0u64;
    let mut used_bits = 0usize;

    for entry in entries_iter {
        let value = (entry as u64) & mask;
        word |= value << used_bits;
        used_bits += bpe;

        if used_bits >= 64 {
            packed_data.push(word);
            used_bits -= 64;
            word = if used_bits == 0 {
                0
            } else {
                value >> (bpe - used_bits)
            };
        }
    }

    if used_bits > 0 {
        packed_data.push(word);
    }

    packed_data
}

#[cfg(test)]
mod tests {
    use crate::pack_direct::{pack_compact, pack_direct};

    #[test]
    fn should_pack_five_bytes() {
        // Given
        let entries: Vec<u32> = vec![
            1, 2, 2, 3, 4, 4, 5, 6, 6, 4, 8, 0, 7, 4, 3, 13, 15, 16, 9, 14, 10, 12, 0, 2,
        ];
        let expected_longs = vec![0x0020863148418841u64, 0x01018A7260F68C87u64];
        let bits_per_entry = 5;

        // When
        let result = pack_direct(entries.into_iter(), bits_per_entry);

        // Then
        assert_eq!(expected_longs, result);
    }

    #[test]
    fn compact_should_split_entries_across_long_boundaries() {
        let entries = vec![
            1, 2, 2, 3, 4, 4, 5, 6, 6, 4, 8, 0, 31, 4, 3, 13, 15, 16, 9, 14, 10, 12, 0, 2,
        ];

        let result = pack_compact(entries.into_iter(), 5);

        assert_eq!(vec![0xF020863148418841, 0x001018A7260F68C9], result);
        assert_eq!(1, (result[0] >> 63) & 1);
        assert_eq!(1, result[1] & 1);
    }

    #[test]
    fn compact_should_not_add_a_long_at_an_exact_boundary() {
        let entries = vec![u32::MAX; 16];

        let result = pack_compact(entries.into_iter(), 4);

        assert_eq!(vec![u64::MAX], result);
    }

    #[test]
    fn compact_should_mask_entries_to_the_configured_width() {
        let result = pack_compact([0x3f, 0x20].into_iter(), 5);

        assert_eq!(vec![0x1f], result);
    }

    #[test]
    fn compact_should_return_empty_output_for_empty_input() {
        let result = pack_compact(std::iter::empty(), 5);

        assert!(result.is_empty());
    }
}
