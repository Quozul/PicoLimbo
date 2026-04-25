use minecraft_protocol::prelude::*;

pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn z(&self) -> i32 {
        self.z
    }
}

impl EncodePacket for BlockPos {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        const PACKED_HORIZONTAL_LENGTH: i32 =
            1 + log2(smallest_encompassing_power_of_two(30_000_000));
        const PACKED_Y_LENGTH: i32 = 64 - 2 * PACKED_HORIZONTAL_LENGTH;
        const PACKED_X_MASK: u64 = (1u64 << PACKED_HORIZONTAL_LENGTH) - 1;
        const PACKED_Y_MASK: u64 = (1u64 << PACKED_Y_LENGTH) - 1;
        const PACKED_Z_MASK: u64 = (1u64 << PACKED_HORIZONTAL_LENGTH) - 1;
        const Y_OFFSET: i32 = 0;
        const Z_OFFSET: i32 = PACKED_Y_LENGTH;
        const X_OFFSET: i32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;
        let mut l: u64 = 0;
        l |= ((self.x as u64) & PACKED_X_MASK) << X_OFFSET;
        l |= ((self.y as u64) & PACKED_Y_MASK) << Y_OFFSET;
        l |= ((self.z as u64) & PACKED_Z_MASK) << Z_OFFSET;

        writer.write(&l)?;
        Ok(())
    }
}

impl DecodePacket for BlockPos {
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        const PACKED_HORIZONTAL_LENGTH: i32 =
            1 + log2(smallest_encompassing_power_of_two(30_000_000));
        const PACKED_Y_LENGTH: i32 = 64 - 2 * PACKED_HORIZONTAL_LENGTH;
        const PACKED_X_MASK: u64 = (1u64 << PACKED_HORIZONTAL_LENGTH) - 1;
        const PACKED_Y_MASK: u64 = (1u64 << PACKED_Y_LENGTH) - 1;
        const PACKED_Z_MASK: u64 = (1u64 << PACKED_HORIZONTAL_LENGTH) - 1;
        const Y_OFFSET: i32 = 0;
        const Z_OFFSET: i32 = PACKED_Y_LENGTH;
        const X_OFFSET: i32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;

        let l: u64 = reader.read()?;
        let x = ((l >> X_OFFSET) & PACKED_X_MASK) as i32;
        let y = ((l >> Y_OFFSET) & PACKED_Y_MASK) as i32;
        let z = ((l >> Z_OFFSET) & PACKED_Z_MASK) as i32;

        Ok(Self { x, y, z })
    }
}

const fn smallest_encompassing_power_of_two(n: i32) -> i32 {
    let mut x = n;
    x -= 1;
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);
    x + 1
}

const fn log2(n: i32) -> i32 {
    if n == 0 {
        return -1;
    }
    31 - (n as u32).leading_zeros() as i32
}
