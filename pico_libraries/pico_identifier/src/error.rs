use thiserror::Error;

/// Errors that can occur while turning a raw string into an `Identifier`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentifierParseError {
    #[error("expected \"namespace:thing\" (missing ':')")]
    MissingColon,

    #[error("namespace must not be empty")]
    EmptyNamespace,

    #[error("thing must not be empty")]
    EmptyThing,

    #[error(
        "illegal character '{ch}' at position {pos} in namespace \"{namespace}\" \
        (allowed: [0-9a-z_.-#])"
    )]
    InvalidNamespaceChar {
        ch: char,
        pos: usize,
        namespace: String,
    },

    #[error(
        "illegal character '{ch}' at position {pos} in thing \"{thing}\" \
        (allowed: [0-9a-z_.-/])"
    )]
    InvalidThingChar { ch: char, pos: usize, thing: String },
}
