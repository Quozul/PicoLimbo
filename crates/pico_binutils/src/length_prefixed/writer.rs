use crate::binary_writer::WriteBytes;
use crate::prelude::{BinaryWriter, BinaryWriterError, Prefixed};

pub trait WriteLengthPrefix: Sized + WriteBytes {
    fn write_usize(writer: &mut BinaryWriter, len: usize) -> Result<(), BinaryWriterError>;
}

fn write_slice<L, T>(writer: &mut BinaryWriter, slice: &[T]) -> Result<(), BinaryWriterError>
where
    L: WriteLengthPrefix,
    T: WriteBytes,
{
    L::write_usize(writer, slice.len())?;
    for item in slice {
        item.write(writer)?;
    }
    Ok(())
}

impl<L, T> WriteBytes for Prefixed<L, &[T]>
where
    L: WriteLengthPrefix,
    T: WriteBytes,
{
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        write_slice::<L, T>(writer, self.0)
    }
}

impl<L, T> WriteBytes for Prefixed<L, Vec<T>>
where
    L: WriteLengthPrefix,
    T: WriteBytes,
{
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        write_slice::<L, T>(writer, self.0.as_slice())
    }
}

impl<L> WriteBytes for Prefixed<L, String>
where
    L: WriteLengthPrefix,
{
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        write_slice::<L, u8>(writer, self.0.as_bytes())
    }
}

impl<L, T> WriteBytes for Prefixed<L, &Vec<T>>
where
    L: WriteLengthPrefix,
    T: WriteBytes,
{
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        write_slice::<L, T>(writer, self.0.as_slice())
    }
}

pub(crate) fn get_i32(len: usize) -> Result<i32, BinaryWriterError> {
    len.try_into().map_err(|_| {
        BinaryWriterError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Length exceeds i32::MAX",
        ))
    })
}

impl WriteLengthPrefix for i32 {
    fn write_usize(writer: &mut BinaryWriter, len: usize) -> Result<(), BinaryWriterError> {
        let len_i32 = get_i32(len)?;
        writer.write(&len_i32)
    }
}

impl WriteLengthPrefix for i16 {
    fn write_usize(writer: &mut BinaryWriter, len: usize) -> Result<(), BinaryWriterError> {
        let len_i16: i16 = len.try_into().map_err(|_| {
            BinaryWriterError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Length exceeds i16::MAX",
            ))
        })?;
        writer.write(&len_i16)
    }
}
