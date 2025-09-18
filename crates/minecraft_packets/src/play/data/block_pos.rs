use minecraft_protocol::prelude::*;

pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl EncodePacket for BlockPos {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let packed_horizontal_length: i32 =
            1 + log2(smallest_encompassing_power_of_two(30_000_000));
        let packed_y_length: i32 = 64 - 2 * packed_horizontal_length;
        let packed_x_mask: u64 = (1u64 << packed_horizontal_length) - 1;
        let packed_y_mask: u64 = (1u64 << packed_y_length) - 1;
        let packed_z_mask: u64 = (1u64 << packed_horizontal_length) - 1;
        let y_offset: i32 = 0;
        let z_offset: i32 = packed_y_length;
        let x_offset: i32 = packed_y_length + packed_horizontal_length;
        let mut l: u64 = 0;
        l |= ((self.x as u64) & packed_x_mask) << x_offset;
        l |= ((self.y as u64) & packed_y_mask) << y_offset;
        l |= ((self.z as u64) & packed_z_mask) << z_offset;

        writer.write(&l)?;
        Ok(())
    }
}

fn smallest_encompassing_power_of_two(n: i32) -> i32 {
    let mut x = n;
    x -= 1;
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);
    x + 1
}

fn log2(n: i32) -> i32 {
    if n == 0 {
        return -1;
    }
    31 - (n as u32).leading_zeros() as i32
}
