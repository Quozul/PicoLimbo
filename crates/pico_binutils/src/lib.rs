extern crate core;

#[cfg(feature = "binary_reader")]
mod binary_reader;
#[cfg(feature = "binary_writer")]
mod binary_writer;
#[cfg(feature = "length_prefixed")]
mod length_prefixed;
#[cfg(feature = "string_indexer")]
mod string_indexer;
#[cfg(feature = "uuid")]
mod uuid;
#[cfg(feature = "var_int")]
mod var_int;

pub mod prelude {
    #[cfg(feature = "binary_reader")]
    pub use crate::binary_reader::{BinaryReader, BinaryReaderError, ReadBytes};
    #[cfg(feature = "binary_writer")]
    pub use crate::binary_writer::{BinaryWriter, BinaryWriterError, WriteBytes};
    #[cfg(feature = "length_prefixed")]
    pub use crate::length_prefixed::prefixed::{IntPrefixed, Prefixed, UShortPrefixed};
    #[cfg(feature = "length_prefixed")]
    pub use crate::length_prefixed::reader::ReadLengthPrefix;
    #[cfg(all(feature = "length_prefixed", feature = "var_int"))]
    pub use crate::length_prefixed::var_int::{VarIntPrefixed, VarIntPrefixedString};
    #[cfg(feature = "length_prefixed")]
    pub use crate::length_prefixed::writer::WriteLengthPrefix;
    #[cfg(feature = "string_indexer")]
    pub use crate::string_indexer::indexer::StringIndexer;
    #[cfg(feature = "var_int")]
    pub use crate::var_int::VarInt;
}
