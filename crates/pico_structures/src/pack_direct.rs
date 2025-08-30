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

#[cfg(test)]
mod tests {
    use crate::pack_direct::pack_direct;

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
}
