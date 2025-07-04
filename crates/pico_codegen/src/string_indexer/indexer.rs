use std::collections::HashSet;

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
