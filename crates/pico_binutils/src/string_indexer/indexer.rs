use crate::binary_reader::BinaryReaderError;
use crate::binary_writer::BinaryWriterError;
use crate::prelude::{BinaryReader, BinaryWriter, VarIntPrefixed};
use std::collections::HashSet;

#[derive(Clone, Default)]
pub struct StringIndexer {
    pub(crate) strings: Vec<String>,
}

impl StringIndexer {
    /// Create a new StringIndexer from any iterator that yields strings
    pub fn new<I, S>(strings: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let unique_strings: HashSet<String> = strings.into_iter().map(|s| s.into()).collect();
        let mut strings_vec: Vec<String> = unique_strings.into_iter().collect();
        strings_vec.sort();
        Self {
            strings: strings_vec,
        }
    }

    /// Returns the index of the string from the indexer
    pub fn get_index(&self, string: &str) -> Option<u16> {
        self.strings
            .iter()
            .position(|s| s == string)
            .map(|idx| idx as u16)
    }

    /// Returns the string given an index
    pub fn get_string(&self, index: u16) -> Option<&str> {
        self.strings.get(index as usize).map(|s| s.as_str())
    }

    /// Get all strings as a slice
    pub fn strings(&self) -> &[String] {
        &self.strings
    }

    #[cfg(feature = "binary_writer")]
    pub fn to_bytes(&self) -> Result<Vec<u8>, BinaryWriterError> {
        let mut writer = BinaryWriter::default();
        let strings = self
            .strings
            .iter()
            .map(VarIntPrefixed::string)
            .collect::<Vec<_>>();
        let prefixed = VarIntPrefixed::new(strings);
        writer.write(&prefixed)?;
        Ok(writer.into_inner())
    }

    #[cfg(feature = "binary_reader")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryReaderError> {
        let mut reader = BinaryReader::new(bytes);
        let strings = reader
            .read::<VarIntPrefixed<Vec<VarIntPrefixed<String>>>>()?
            .into_inner()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        Ok(Self { strings })
    }
}

impl StringIndexer {
    /// Create from a slice of string-like items
    pub fn from_slice<S: Into<String> + Clone>(strings: &[S]) -> Self {
        Self::new(strings.iter().cloned().map(|s| s.into()))
    }

    /// Create from a Vec of string-like items
    pub fn from_vec<S: Into<String>>(strings: Vec<S>) -> Self {
        Self::new(strings.into_iter().map(|s| s.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::StringIndexer;

    #[test]
    fn test_to_bytes() {
        // Given
        let strings = vec!["a", "b"];
        let indexer = StringIndexer::new(strings);

        // When
        let bytes = indexer.to_bytes().unwrap();

        // Then
        assert_eq!(bytes, &[2, 1, 97, 1, 98]);
    }

    #[test]
    fn test_from_bytes() {
        // Given
        let bytes = &[2, 1, 97, 1, 98];

        // When
        let indexer = StringIndexer::from_bytes(bytes).unwrap();

        // Then
        assert_eq!(indexer.strings, vec!["a", "b"]);
    }
}
