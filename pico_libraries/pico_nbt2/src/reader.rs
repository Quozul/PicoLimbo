use crate::de::NbtReader;
use crate::{Error, NbtOptions, Value};
use byteorder::ReadBytesExt;
use serde::de;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

/// Deserializes NBT data from a reader.
///
/// Returns a tuple of the root tag name and the deserialized NBT value.
///
/// # Errors
/// Returns an error if reading fails or the data is malformed.
pub fn from_reader<R: Read>(reader: R) -> crate::Result<(String, Value)> {
    from_reader_with_options(reader, NbtOptions::default())
}

/// Deserializes NBT data from a reader with options.
///
/// # Errors
/// Returns an error if reading fails or the data is malformed.
pub fn from_reader_with_options<R: Read>(
    reader: R,
    options: NbtOptions,
) -> crate::Result<(String, Value)> {
    let mut decoder = NbtReader::new_with_options(reader, options);
    decoder.read_root()
}

/// Deserializes NBT data from a byte slice.
///
/// Returns a tuple of the root tag name and the deserialized NBT value.
///
/// # Errors
/// Returns an error if the data is malformed.
pub fn from_slice(bytes: &[u8]) -> crate::Result<(String, Value)> {
    from_reader(io::Cursor::new(bytes))
}

/// Deserializes NBT data from a byte slice with options.
///
/// # Errors
/// Returns an error if the data is malformed.
pub fn from_slice_with_options(
    bytes: &[u8],
    options: NbtOptions,
) -> crate::Result<(String, Value)> {
    from_reader_with_options(io::Cursor::new(bytes), options)
}

/// Deserializes NBT data from a file.
///
/// This function automatically detects compression (Gzip or Zlib) and decodes it.
///
/// # Errors
/// Returns an error if reading fails or the data is malformed.
pub fn from_file(file: &mut File) -> crate::Result<(String, Value)> {
    let reader = crate::io::decode(file)?;
    from_reader(reader)
}

/// Deserializes NBT data from a file with options.
///
/// This function automatically detects compression (Gzip or Zlib) and decodes it.
///
/// # Errors
/// Returns an error if reading fails or the data is malformed.
pub fn from_file_with_options(
    file: &mut File,
    options: NbtOptions,
) -> crate::Result<(String, Value)> {
    let reader = crate::io::decode(file)?;
    from_reader_with_options(reader, options)
}

/// Deserializes NBT data from a file path.
///
/// This function opens the file and automatically detects compression.
///
/// # Errors
/// Returns an error if opening the file fails, reading fails, or the data is malformed.
pub fn from_path<P: AsRef<Path>>(path: P) -> crate::Result<(String, Value)> {
    let mut file = File::open(path)?;
    from_file(&mut file)
}

/// Deserializes NBT data from a file path with options.
///
/// This function opens the file and automatically detects compression.
///
/// # Errors
/// Returns an error if opening the file fails, reading fails, or the data is malformed.
pub fn from_path_with_options<P: AsRef<Path>>(
    path: P,
    options: NbtOptions,
) -> crate::Result<(String, Value)> {
    let mut file = File::open(path)?;
    from_file_with_options(&mut file, options)
}

/// Deserializes NBT data from a reader into a Rust struct.
///
/// The root tag must be a compound tag.
///
/// # Errors
/// Returns an error if reading fails, the root tag is not a compound,
/// or deserialization fails.
pub fn from_reader_struct<T: de::DeserializeOwned, R: Read>(
    reader: R,
) -> crate::Result<(String, T)> {
    let mut decoder = NbtReader::new(reader);
    let tag_id = decoder.reader.read_u8()?;
    if tag_id != 10 {
        return Err(Error::UnexpectedTag {
            expected: 10,
            found: tag_id,
        });
    }
    let name = decoder.read_string()?;

    decoder.next_tag_id = Some(10);
    Ok((name, T::deserialize(&mut decoder)?))
}

/// Deserializes NBT data from a file into a Rust struct.
///
/// The root tag must be a compound tag.
///
/// # Errors
/// Returns an error if reading fails, the root tag is not a compound,
/// or deserialization fails.
pub fn from_file_struct<T: de::DeserializeOwned>(file: File) -> crate::Result<(String, T)> {
    let reader = crate::io::decode(file)?;
    from_reader_struct(reader)
}

/// Deserializes NBT data from a path into a Rust struct.
///
/// The root tag must be a compound tag.
///
/// # Errors
/// Returns an error if reading fails, the root tag is not a compound,
/// or deserialization fails.
pub fn from_path_struct<T: de::DeserializeOwned>(path: &Path) -> crate::Result<(String, T)> {
    let file = File::open(path)?;
    from_file_struct(file)
}

/// Deserializes NBT data from a byte slice into a Rust struct.
///
/// The root tag must be a compound tag.
///
/// # Errors
/// Returns an error if the root tag is not a compound or deserialization fails.
pub fn from_slice_struct<T: de::DeserializeOwned>(bytes: &[u8]) -> crate::Result<(String, T)> {
    from_reader_struct(io::Cursor::new(bytes))
}
