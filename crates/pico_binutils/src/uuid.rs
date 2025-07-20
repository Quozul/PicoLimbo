use crate::binary_reader::{BinaryReader, BinaryReaderError, ReadBytes};
use crate::binary_writer::WriteBytes;
use crate::prelude::{BinaryWriter, BinaryWriterError};
use uuid::Uuid;

#[cfg(feature = "binary_reader")]
impl ReadBytes for Uuid {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let mut bytes = [0u8; 16];
        let bytes_read = reader.read_bytes(&mut bytes)?;

        if bytes_read != 16 {
            return Err(BinaryReaderError::UnexpectedEof);
        }

        Ok(Uuid::from_bytes(bytes))
    }
}

#[cfg(feature = "binary_writer")]
impl WriteBytes for Uuid {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        let bytes = self.as_bytes().as_slice().to_vec();
        writer.write_bytes(&bytes)?;
        Ok(())
    }
}
