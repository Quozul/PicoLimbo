extern crate core;

#[cfg(feature = "binary_reader")]
mod binary_reader;
#[cfg(feature = "binary_writer")]
mod binary_writer;
#[cfg(feature = "length_prefixed")]
mod length_prefixed;
#[cfg(feature = "var_int")]
mod var_int;

pub mod prelude {
    #[cfg(feature = "binary_reader")]
    pub use crate::binary_reader::{BinaryReader, BinaryReaderError};
    #[cfg(feature = "binary_writer")]
    pub use crate::binary_writer::{BinaryWriter, BinaryWriterError};
    #[cfg(feature = "length_prefixed")]
    pub use crate::length_prefixed::prefixed::{IntPrefixed, Prefixed, ShortPrefixed};
    #[cfg(all(feature = "length_prefixed", feature = "var_int"))]
    pub use crate::length_prefixed::var_int::VarIntPrefixed;
    #[cfg(feature = "var_int")]
    pub use crate::var_int::VarInt;
}
