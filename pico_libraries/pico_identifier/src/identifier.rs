use std::convert::TryFrom;

use crate::{
    error::IdentifierParseError,
    validation::{is_valid_namespace_char, is_valid_path_char},
};

/// A namespaced identifier in the format `namespace:thing`.
///
/// # Examples
///
/// ```
/// # use pico_identifier::Identifier;
/// let id = Identifier::new("minecraft", "stone")?;
/// assert_eq!(id.namespace, "minecraft");
/// assert_eq!(id.thing, "stone");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub thing: String,
}

impl Identifier {
    /// Creates an identifier without validation.
    pub fn new_unchecked<N, T>(namespace: N, thing: T) -> Self
    where
        N: Into<String>,
        T: Into<String>,
    {
        Self {
            namespace: namespace.into(),
            thing: thing.into(),
        }
    }

    /// Creates an identifier, validating namespace and thing characters.
    ///
    /// # Errors
    ///
    /// Returns an error if either field is empty or contains invalid characters.
    pub fn new<N, T>(namespace: N, thing: T) -> Result<Self, IdentifierParseError>
    where
        N: Into<String>,
        T: Into<String>,
    {
        let namespace = namespace.into();
        let thing = thing.into();
        if namespace.is_empty() {
            return Err(IdentifierParseError::EmptyNamespace);
        }
        if thing.is_empty() {
            return Err(IdentifierParseError::EmptyThing);
        }

        for (idx, ch) in namespace.chars().enumerate() {
            if !is_valid_namespace_char(ch) {
                return Err(IdentifierParseError::InvalidNamespaceChar {
                    ch,
                    pos: idx,
                    namespace: namespace.clone(),
                });
            }
        }

        for (idx, ch) in thing.chars().enumerate() {
            if !is_valid_path_char(ch) {
                return Err(IdentifierParseError::InvalidThingChar {
                    ch,
                    pos: idx,
                    thing: thing.clone(),
                });
            }
        }

        Ok(Self { namespace, thing })
    }

    /// Creates a `minecraft` namespace identifier without validation.
    pub fn vanilla_unchecked<T>(thing: T) -> Self
    where
        T: Into<String>,
    {
        Self::new_unchecked("minecraft", thing)
    }

    /// Creates a `minecraft` namespace identifier with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if either field is empty or contains invalid characters.
    pub fn vanilla<T>(thing: T) -> Result<Self, IdentifierParseError>
    where
        T: Into<String>,
    {
        Self::new("minecraft", thing)
    }

    #[must_use]
    pub fn is_tag(&self) -> bool {
        self.namespace.starts_with('#')
    }

    #[must_use]
    pub fn normalize(&self) -> Self {
        if self.is_tag() {
            Self::new_unchecked(&self.namespace[1..], &self.thing)
        } else {
            self.clone()
        }
    }
}

impl TryFrom<&str> for Identifier {
    type Error = IdentifierParseError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let mut parts = v.splitn(2, ':');
        let namespace = parts.next().ok_or(IdentifierParseError::MissingColon)?;
        let thing = parts.next().ok_or(IdentifierParseError::MissingColon)?;
        Self::new(namespace, thing)
    }
}
