use crate::prelude::*;

/// A set of bits, encoded as a variable-length array of longs on the wire.
///
/// Wire format history:
/// - Up to 26.2 (` FriendlyByteBuf.readBitSet`): VarInt long count followed by
///   big-endian longs (`BitSet.valueOf(long[])`).
/// - 26.3 and later (`ByteBufCodecs.BIT_SET`): VarInt byte count followed
///   by a byte array where byte `i` holds bits `8i..8i+7`
///   (`BitSet.valueOf(byte[])`, see `BitSet.toByteArray()`).
#[derive(Default, Clone)]
pub struct BitSet {
    data: LengthPaddedVec<i64>,
}

impl BitSet {
    pub fn new(data: Vec<i64>) -> Self {
        Self {
            data: LengthPaddedVec::new(data),
        }
    }

    /// Encodes the bitset in the pre-26.3 format: VarInt long count followed by
    /// big-endian longs.
    fn encode_as_longs(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.data.encode(writer, protocol_version)
    }

    /// Encodes the bitset in the 26.3+ format: VarInt byte count followed
    /// by the minimal little-endian byte array, matching Java's
    /// `BitSet.toByteArray()`.
    fn encode_as_bytes(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let mut bytes = Vec::new();
        for long in self.data.inner() {
            bytes.extend_from_slice(&long.to_le_bytes());
        }
        // Java's BitSet.toByteArray() returns the minimal number of bytes.
        while bytes.last() == Some(&0) {
            bytes.pop();
        }
        let length = VarInt::new(bytes.len() as i32);
        length.encode(writer, protocol_version)?;
        writer.write_bytes(&bytes)?;
        Ok(())
    }
}

impl EncodePacket for BitSet {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V26_3) {
            self.encode_as_bytes(writer, protocol_version)
        } else {
            self.encode_as_longs(writer, protocol_version)
        }
    }
}

impl DecodePacket for BitSet {
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V26_3) {
            let length = VarInt::decode(reader, protocol_version)?.inner() as usize;
            let mut bytes = vec![0u8; length];
            reader.read_bytes(&mut bytes)?;

            let long_count = bytes.len().div_ceil(8);
            let mut longs = Vec::with_capacity(long_count);
            for index in 0..long_count {
                let mut chunk = [0u8; 8];
                let end = ((index + 1) * 8).min(bytes.len());
                chunk[..end - index * 8].copy_from_slice(&bytes[index * 8..end]);
                longs.push(i64::from_le_bytes(chunk));
            }
            Ok(Self::new(longs))
        } else {
            LengthPaddedVec::<i64>::decode(reader, protocol_version).map(|data| Self { data })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V26_2: ProtocolVersion = ProtocolVersion::V26_2;
    const V26_3: ProtocolVersion = ProtocolVersion::V26_3;

    #[test]
    fn encode_long_format_before_26_3() {
        let bitset = BitSet::new(vec![0x0000_0000_03FF_FFFF]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_2).unwrap();
        // VarInt long count (1), then the big-endian long.
        assert_eq!(
            writer.as_slice(),
            &[0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0xFF, 0xFF, 0xFF]
        );
    }

    #[test]
    fn encode_byte_format_since_26_3() {
        let bitset = BitSet::new(vec![0x0000_0000_03FF_FFFF]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_3).unwrap();
        // VarInt byte count (4), then the minimal little-endian byte array.
        assert_eq!(writer.as_slice(), &[0x04, 0xFF, 0xFF, 0xFF, 0x03]);
    }

    #[test]
    fn encode_empty_bitset_since_26_3() {
        let bitset = BitSet::new(vec![]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_3).unwrap();
        assert_eq!(writer.as_slice(), &[0x00]);
    }

    #[test]
    fn encode_empty_bitset_before_26_3() {
        let bitset = BitSet::new(vec![]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_2).unwrap();
        assert_eq!(writer.as_slice(), &[0x00]);
    }

    #[test]
    fn encode_multiple_longs_since_26_3() {
        // Bit 63 and bit 64 set => 0x8000_0000_0000_0001 and 0x1.
        let bitset = BitSet::new(vec![0x8000_0000_0000_0001u64 as i64, 0x1]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_3).unwrap();
        // Bit 0 => byte 0 => 01, bit 63 => byte 7 => 80, bit 64 => byte 8 => 01.
        // Minimal representation trims the trailing zero bytes of the second long.
        assert_eq!(
            writer.as_slice(),
            &[0x09, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x01]
        );
    }

    #[test]
    fn roundtrip_byte_format_since_26_3() {
        let bitset = BitSet::new(vec![0x0000_0000_03FF_FFFF, 0x0000_0000_0000_0001]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_3).unwrap();

        let mut reader = BinaryReader::new(writer.as_slice());
        let decoded = BitSet::decode(&mut reader, V26_3).unwrap();
        assert_eq!(decoded.data.inner(), bitset.data.inner());
    }

    #[test]
    fn roundtrip_long_format_before_26_3() {
        let bitset = BitSet::new(vec![0x0000_0000_03FF_FFFF]);
        let mut writer = BinaryWriter::default();
        bitset.encode(&mut writer, V26_2).unwrap();

        let mut reader = BinaryReader::new(writer.as_slice());
        let decoded = BitSet::decode(&mut reader, V26_2).unwrap();
        assert_eq!(decoded.data.inner(), bitset.data.inner());
    }
}
