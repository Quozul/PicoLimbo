#[cfg(feature = "binary_reader")]
pub mod binary_reader;
#[cfg(feature = "binary_writer")]
mod binary_writer;
#[cfg(feature = "string_indexer")]
mod string_indexer;

pub mod prelude {
    #[cfg(feature = "binary_reader")]
    pub use crate::binary_reader::BinaryReader;
    #[cfg(feature = "binary_writer")]
    pub use crate::binary_writer::BinaryWriter;
    #[cfg(feature = "string_indexer")]
    pub use crate::string_indexer::string_indexer::StringIndexer;
}
