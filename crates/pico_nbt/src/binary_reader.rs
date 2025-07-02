use std::string::FromUtf8Error;

macro_rules! impl_read_number {
    ($fn_name:ident, $type:ty) => {
        pub fn $fn_name(&mut self) -> $type {
            let size = std::mem::size_of::<$type>();
            let bytes = &self.raw[self.index..self.index + size];
            let integer = <$type>::from_be_bytes(bytes.try_into().unwrap());
            self.index += size;
            integer
        }
    };
}

macro_rules! impl_read_array {
    ($fn_name:ident, $type:ty, $reader:ident) => {
        pub fn $fn_name(&mut self) -> Vec<$type> {
            let size = self.read_i32() as usize;
            let mut values = Vec::with_capacity(size);

            for _ in 0..size {
                let next_tag = self.$reader();
                values.push(next_tag);
            }

            values
        }
    };
}

pub struct BinaryReader<'a> {
    raw: &'a [u8],
    index: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(raw: &'a [u8]) -> Self {
        Self { raw, index: 0 }
    }

    pub fn read_string(&mut self) -> Result<String, FromUtf8Error> {
        let size = self.read_u16() as usize;
        let bytes = &self.raw[self.index..self.index + size];
        self.index += size;
        String::from_utf8(bytes.to_vec())
    }

    pub fn read_name(&mut self) -> Option<String> {
        self.read_string().ok().filter(|s| !s.is_empty())
    }

    pub fn read_type(&mut self) -> u8 {
        self.read_u8()
    }

    impl_read_number!(read_i8, i8);
    impl_read_number!(read_u8, u8);
    impl_read_number!(read_i16, i16);
    impl_read_number!(read_u16, u16);
    impl_read_number!(read_i32, i32);
    impl_read_number!(read_i64, i64);
    impl_read_number!(read_f32, f32);
    impl_read_number!(read_f64, f64);
    impl_read_array!(read_byte_array, i8, read_i8);
    impl_read_array!(read_int_array, i32, read_i32);
    impl_read_array!(read_long_array, i64, read_i64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_i8() {
        let data = [0x7F];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_i8(), 127);
    }

    #[test]
    fn test_read_i16() {
        let data = [0x7F, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_i16(), 32767);
    }

    #[test]
    fn test_read_u16() {
        let data = [0x0F, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_u16(), 4095);
    }

    #[test]
    fn test_read_i32() {
        let data = [0x7F, 0xFF, 0xFF, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_i32(), 2147483647);
    }

    #[test]
    fn test_read_f32() {
        let data = [0x3F, 0x80, 0x00, 0x00];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_f32(), 1.0);
    }

    #[test]
    fn test_read_string() {
        let data = [0, 5, 72, 69, 76, 76, 79];
        let mut reader = BinaryReader::new(&data);
        let parsed = reader.read_string().unwrap();

        assert_eq!(parsed, "HELLO");
    }
}
