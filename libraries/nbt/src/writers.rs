fn size_to_u16_bytes(size: usize) -> [u8; 2] {
    (size as u16).to_be_bytes()
}

pub fn size_to_i32_bytes(size: usize) -> [u8; 4] {
    (size as i32).to_be_bytes()
}

fn create_initial_buffer<T>(size: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(size * size_of::<T>() + 4);
    buffer.extend_from_slice(&size_to_i32_bytes(size));
    buffer
}

pub fn write_string(input: String) -> Vec<u8> {
    let input_bytes = input.as_bytes();
    let mut buffer = Vec::with_capacity(input_bytes.len() + 2);
    buffer.extend_from_slice(&size_to_u16_bytes(input.len()));
    buffer.extend(input_bytes);
    buffer
}

macro_rules! impl_write_array {
    ($fn_name:ident, $type:ty) => {
        pub fn $fn_name(input: &[$type]) -> Vec<u8> {
            let mut buffer = create_initial_buffer::<$type>(input.len());
            let bytes = input
                .iter()
                .flat_map(|e| e.to_be_bytes())
                .collect::<Vec<u8>>();
            buffer.extend(bytes);
            buffer
        }
    };
}

impl_write_array!(write_array_i8, i8);
impl_write_array!(write_array_i32, i32);
impl_write_array!(write_array_i64, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write() {
        let parsed = write_string("HELLO".to_string());

        assert_eq!(parsed, &[0, 5, 72, 69, 76, 76, 79]);
    }

    #[test]
    fn test_write_array_i8() {
        let parsed = write_array_i8(&[1, 2, 3, 4, 5]);

        assert_eq!(parsed, &[0, 0, 0, 5, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_write_array_i32() {
        let parsed = write_array_i32(&[1, 2, 3, 4, 5]);

        assert_eq!(
            parsed,
            &[0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5]
        );
    }

    #[test]
    fn test_write_array_i64() {
        let parsed = write_array_i64(&[1, 2, 3, 4, 5]);

        assert_eq!(
            parsed,
            &[
                0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3,
                0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5
            ]
        );
    }
}
