//! # identifier
//!
//! A tiny library that offers a validated `Identifier` type (`namespace:thing`)
//! together with convenient conversion and Serde support.

#[cfg(feature = "serde")]
mod serde_impl;

mod error;
mod identifier;
mod identifier_fmt;
mod validation;

pub use error::IdentifierParseError;
pub use identifier::Identifier;

/// A tiny “prelude” so downstream crates can write:
///
/// ```rust
/// use pico_identifier::prelude::*;
/// ```
pub mod prelude {
    pub use super::Identifier;
    pub use super::IdentifierParseError;
}
