pub(crate) trait WriteBytes {
    fn write(&self, out: &mut Vec<u8>);
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl WriteBytes for $t {
                #[inline]
                fn write(&self, out: &mut Vec<u8>) {
                    out.extend_from_slice(&self.to_be_bytes());
                }
            }
        )*
    }
}

impl_int!(u8, i8, i16, i32, i64, f32, f64);

impl<T: WriteBytes> WriteBytes for [T] {
    #[inline]
    fn write(&self, out: &mut Vec<u8>) {
        let length = self.len() as i32;
        length.write(out);
        for elt in self {
            elt.write(out);
        }
    }
}

impl<T: WriteBytes> WriteBytes for Vec<T> {
    #[inline]
    fn write(&self, out: &mut Vec<u8>) {
        self.as_slice().write(out)
    }
}

impl WriteBytes for String {
    #[inline]
    fn write(&self, out: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        let length = bytes.len() as i16;
        length.write(out);
        out.extend_from_slice(bytes);
    }
}

impl<T: WriteBytes + ?Sized> WriteBytes for &T {
    #[inline]
    fn write(&self, out: &mut Vec<u8>) {
        (*self).write(out)
    }
}

pub(crate) struct BinaryWriter(Vec<u8>);

impl BinaryWriter {
    pub(crate) fn new() -> Self {
        Self(Vec::with_capacity(1024))
    }

    pub(crate) fn write<T: WriteBytes>(&mut self, v: T) {
        v.write(&mut self.0);
    }

    pub(crate) fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_byte() {
        // Given
        let mut writer = BinaryWriter::new();

        // When
        writer.write(0_u8);

        // Then
        assert_eq!(writer.into_inner(), vec![0]);
    }

    #[test]
    fn test_string() {
        // Given
        let mut writer = BinaryWriter::new();
        let input = "hello world".to_string();

        // When
        writer.write(input);

        // Then
        assert_eq!(
            writer.into_inner(),
            vec![
                0, 11, // String length
                104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100 // String content
            ]
        );
    }

    #[test]
    fn test_vec() {
        // Given
        let mut writer = BinaryWriter::new();
        let input = vec![1_u8, 2, 3];

        // When
        writer.write(input);

        // Then
        assert_eq!(
            writer.into_inner(),
            vec![
                0, 0, 0, 3, // Vec length
                1, 2, 3 // Data
            ]
        );
    }
}
